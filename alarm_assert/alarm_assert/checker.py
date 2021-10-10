import pika
from collections import namedtuple 
import json

class FailedAssertion(Exception):
    pass

class CheckerCase:
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

class CheckerExecutor:
   
    def __init__(self, rule, alarm, datasets):
        self.rule = rule
        self.alarm = alarm
        self.datasets = datasets
    
    def exec(self):
        self.alarm.check(self.rule, self.fetch_datasets())

    def fetch_datasets(self):
        data = {}
        for dataset in self.datasets:
            data[dataset.identity()] = dataset.fetch()
        return data

class Alarm:
    
    def alarm(self, exception):
        raise NotImplementedError

    def succeeds(self, rule):
        raise NotImplementedError

    def check(self, rule, datasets):
        try:
            exec(rule, {"datasets": datasets, 
                        "FailedAssertion": FailedAssertion,
                        "CheckerCase": CheckerCase})
            self.succeeds(rule) # we know what is the rule, we probably want to emitt with an id as well
        except FailedAssertion as e:
            self.alarm(e)

class AlarmEventProducer(Alarm):
    """This Alarm implementation sends a message to {topic} when the alarm fails or succeeds.
    For now the implementation only supports rabbitmq"""

    """
    event = {
        id: alarm_id,
        result: succeeded|failed
        error: Optional[str]
        rule: str
    }
    """

    SUCCEEDED_RESULT = 'succeeded'
    FAILED_RESULT = 'failed'
    
    Event = namedtuple('Event', 'id result error rule')

    def __init__(self, id, succeeded_topic, failed_topic, queue_connector):
        self.id = id
        self.succeeded_topic = succeeded_topic
        self.failed_topic = failed_topic
        self.queue_connector = queue_connector

    def succeeds(self, rule):
        self.emit(self.succeeded_topic, self.event(None, rule))
    
    def alarm(self, exception):
        self.emit(self.failed_topic, self.event(repr(exception), None)) # TODO pass the rule as well

    def event(self, error, rule):
        result = AlarmEventProducer.SUCCEEDED_RESULT
        if error is not None:
            result = AlarmEventProducer.FAILED_RESULT
        return AlarmEventProducer.Event(self.id, result, error, rule)

    def emit(self, topic, event):
        self.queue_connector.publish(topic, event)

class RabbitMQConnector():

    ConnectionParams = namedtuple('ConnectionParams', 'host')

    def __init__(self, connection_params):
        self.connection = pika.BlockingConnection(pika.ConnectionParameters(connection_params.host)) # pika.ConnectionParameters()
        self.channel = self.connection.channel()
    
    def publish(self, topic, event):
        self.channel.basic_publish(exchange='', routing_key=topic, body=json.dumps(event))