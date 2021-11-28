import pika
from collections import namedtuple
import json
import os
import sqlite3
import ast

from configparser import ConfigParser


class FailedAssertion(Exception):
    pass


class CheckerCase:

    def assertEmpty(self, collection, message):
        self.assertLessThan(collection, 1, message)

    def assertEqual(self, first_item, second_item, message):
        self.assertTrue(first_item == second_item, message)

    def assertLessThan(self, collection, max_size, message):
        self.assertTrue(len(collection) < max_size, message)

    def assertGreaterThan(self, collection, size, message):
        self.assertTrue(len(collection) > size, message)

    def assertFalse(self, expression, message):
        self.assertTrue(not expression, message)

    def assertTrue(self, expression, message):
        if not expression:
            raise FailedAssertion(message)


class Dataset:

    def identity(self):
        """The id or name of your dataset, this will be used by the rules"""
        pass

    def fetch(self):
        """Fetch does whatever is needed to get the data, if it's a SQL dataset, it will call the database
        and return an array with the results"""
        pass


class SQLiteDataset(Dataset):

    def __init__(self, conn, id, sql):
        self.id = id
        self.sql = sql
        self.conn = conn

    def identity(self):
        return self.id

    def fetch(self):
        cur = self.conn.cursor()
        return cur.execute(self.sql).fetchall()


class CheckerExecutor:

    def __init__(self, rule, alarm, datasets):
        self.rule = rule
        self.alarm = alarm
        self.datasets = datasets

    def exec(self):
        return self.alarm.check(self.rule, self.fetch_datasets())

    def fetch_datasets(self):
        data = {}
        for dataset in self.datasets:
            data[dataset.identity()] = dataset.fetch()
        return data


class Alarm:

    def alarm(self, exception, datasets):
        raise NotImplementedError

    def succeeds(self, rule, datasets):
        raise NotImplementedError

    def check(self, rule, datasets):
        try:
            parsed = ParseCheckerCode(rule).removed_unwanted_nodes()

            exec(compile(parsed, filename="", mode="exec"), {"datasets": datasets,
                                                             "FailedAssertion": FailedAssertion,
                                                             "CheckerCase": CheckerCase})
            self.succeeds(rule, datasets)
            return True
        except Exception as e:
            self.alarm(e, datasets)
            return False


class AlarmEventProducer(Alarm):
    """This Alarm implementation sends a message to {topic} when the alarm fails or succeeds.
    For now the implementation only supports rabbitmq"""

    """
    event = {
        id: alarm_id,
        result: succeeded|failed
        error: Optional[str]
        rule: str,
        datasets: list[list[str]]
    }
    """

    SUCCEEDED_RESULT = 'succeeded'
    FAILED_RESULT = 'failed'

    Event = namedtuple('Event', 'id result error rule dataset')

    def __init__(self, id, succeeded_topic, failed_topic, queue_connector, rule):
        self.id = id
        self.succeeded_topic = succeeded_topic
        self.failed_topic = failed_topic
        self.queue_connector = queue_connector
        self.rule = rule

        # Make sure the queues exist, otherwise messages will be dropped
        self.queue_connector.queue_declare(succeeded_topic)
        self.queue_connector.queue_declare(failed_topic)

    def succeeds(self, rule, datasets):
        self.emit(self.succeeded_topic, self.event(None, rule, datasets))

    def alarm(self, exception, datasets):
        self.emit(self.failed_topic, self.event(
            repr(exception), self.rule, datasets))

    def check(self, rule, datasets):
        return super().check(self.rule, datasets)

    def event(self, error, rule, dataset=None):
        result = AlarmEventProducer.SUCCEEDED_RESULT
        if error is not None:
            result = AlarmEventProducer.FAILED_RESULT
        return AlarmEventProducer.Event(self.id, result, error, rule, dataset)

    def emit(self, topic, event):
        self.queue_connector.publish(topic, json.dumps(event._asdict()))


class RabbitMQConnector():

    ConnectionParams = namedtuple('ConnectionParams', 'host')

    def __init__(self, connection_params):
        self.connection = pika.BlockingConnection(
            pika.ConnectionParameters(connection_params.host))
        self.channel = self.connection.channel()

    def publish(self, topic, event):
        self.channel.basic_publish(
            exchange='', routing_key=topic, body=json.dumps(event))

    def queue_declare(self, queue):
        self.channel.queue_declare(queue=queue)


class ParseCheckerCode(ast.NodeTransformer):

    def __init__(self, code):
        self.tree = ast.parse(code)

    def removed_unwanted_nodes(self):
        return self.visit(self.tree)

    def handle_assigns(self, node):
        ids = [target.id for target in node.targets]
        local_variables = [key for key in globals().keys()]
        for id in ids:
            # If the user is trying to change the value of any globals()
            # they are probably doing something wrong
            if id in local_variables:
                return None
        return node

    def handle_single_assigns(self, node):
        local_variables = [key for key in locals().keys()]

        # If the user is trying to change the value of any globals()
        # they are probably doing something wrong
        if node.target.id in local_variables:
            return None

        return node

    def visit_FunctionDef(self, node):
        return None

    def visit_AsyncFunctionDef(self, node):
        return None

    def visit_ClassDef(self, node):
        return None

    def visit_Assign(self, node):
        return self.handle_assigns(node)

    def visit_AnnAssign(self, node):
        return self.handle_single_assigns(node)

    def visit_AugAssign(self, node):
        return self.handle_single_assigns(node)

    def visit_Import(self, node):
        return None

    def visit_ImportFrom(self, node):
        return None


# Execute this on a cronjob every 5 minutes
if __name__ == '__main__':

    config_object = ConfigParser()

    config_object.read(os.environ['CONFIG'])

    rabbit_mq_env = RabbitMQConnector.ConnectionParams(
        config_object['CHECKER']['RABBITMQ_HOST'])
    rabbitmq_connector = RabbitMQConnector(rabbit_mq_env)

    sqlite_conn = sqlite3.connect(config_object['CHECKER']['SQLITE_PATH'])
    cur = sqlite_conn.cursor()
    cur.execute("SELECT id, desc, status FROM checker")

    checkers = cur.fetchall()

    executors = []

    for checker in checkers:
        alarm = AlarmEventProducer(checker[0], config_object['CHECKER']['SUCCEEDED_TOPIC'],
                                   config_object['CHECKER']['FAILED_TOPIC'], rabbitmq_connector, checker[1])

        cur.execute(
            "SELECT name, code FROM datasource, checker_datasource where checker_id = ? and datasource_id = id", [checker[0]])

        datasources_code = cur.fetchall()

        datasources = []

        for ds_code in datasources_code:
            datasources.append(SQLiteDataset(
                sqlite_conn, ds_code[0], ds_code[1]))

        executors.append(CheckerExecutor(checker[1], alarm, datasources))

    for executor in executors:
        checker_id = executor.alarm.id
        status = 'GREEN'

        succeeded = executor.exec()
        if succeeded:
            status = 'GREEN'
        else:
            status = 'RED'

        cur.execute('UPDATE checker set status = ? where id = ?',
                    [status, checker_id])
        cur.execute(
            "INSERT INTO CHECKER_EXECUTION (checker_id, run_at, status) VALUES(?, strftime('%Y-%m-%d %H:%M:%S', 'now'), ?)", [checker_id, status])

    sqlite_conn.commit()
    sqlite_conn.close()

