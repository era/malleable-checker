import unittest
from unittest.mock import Mock
from unittest.mock import ANY

import alarm_assert.checker as checker

class FakeAlarm(checker.Alarm):
    def alarm(self, exception):
        raise exception
    
    def succeeds(self, _):
        pass

class Dataset(checker.Dataset):
    def __init__(self, name, data):
        self.name = name
        self.data = data
    
    def identity(self):
        return self.name
    
    def fetch(self):
        return self.data

class TestAlarm(unittest.TestCase):
    
    def test_if_alarm_triggers(self):
        rule = "CheckerCase().assertTrue(False, 'Something bad happened')"
        with self.assertRaises(checker.FailedAssertion):
            FakeAlarm().check(rule, [])

    def test_does_not_trigger_alarm(self):
        rule = "CheckerCase().assertTrue(True, 'Something bad happened')"
        FakeAlarm().check(rule, [])


    def test_can_use_datasets_in_rules(self):
        rule = "CheckerCase().assertGreaterThan(datasets['my_data'], 2, 'Something bad happened')"
        FakeAlarm().check(rule, {'my_data': [1,2,3]})

class TestCheckerExecutor(unittest.TestCase):

    def test_if_exec_triggers_alarm(self):
        not_empty = Dataset('my_cool_data', ['123'])
        rule = "CheckerCase().assertLessThan(datasets['my_cool_data'], 0, 'msg')"
        exec = checker.CheckerExecutor(rule, FakeAlarm(), [not_empty])
        with self.assertRaises(checker.FailedAssertion):
            exec.exec()

    def test_if_builds_datasets(self):
        a = Dataset('my_cool_data', ['123'])
        b = not_empty = Dataset('b', ['1234'])
        exec = checker.CheckerExecutor(None, FakeAlarm(), [a,b])
        datasets = exec.fetch_datasets()

        self.assertEqual({a.identity(): a.fetch(),
                          b.identity(): b.fetch()}, datasets)

class TestAlarmEventProducer(unittest.TestCase):

    ALARM_ID = 123
    FAILURE_TOPIC = 'failed'
    SUCCEEDED_TOPIC = 'succeeded'

    def alarm(self, mock_connector, rule):
        return checker.AlarmEventProducer(self.ALARM_ID, self.SUCCEEDED_TOPIC, self.FAILURE_TOPIC, mock_connector, rule)

    def test_if_emit_failure_event(self):
         rule = "CheckerCase().assertTrue(False, 'Something bad happened')"
         connector = Mock()
         self.alarm(connector, rule).check([])
         connector.publish.assert_called_with(self.FAILURE_TOPIC, ANY)

    def test_if_emit_succeeded_event(self):
         rule = "CheckerCase().assertTrue(True, 'Something bad happened')"
         connector = Mock()
         self.alarm(connector, rule).check([])
         connector.publish.assert_called_with(self.SUCCEEDED_TOPIC, ANY)
        

    def test_builds_correctly_succeeded_event(self):
        event = self.alarm(None, 'MY_RULE').event(None, 'MY_RULE')
        self.assertEqual(TestAlarmEventProducer.ALARM_ID, event.id)
        self.assertEqual('MY_RULE', event.rule)
        self.assertEqual(None, event.error)
        self.assertEqual(checker.AlarmEventProducer.SUCCEEDED_RESULT, event.result)

    def test_builds_correctly_failed_event(self):
        exception = Exception('test')
        event = self.alarm(None, 'MY_RULE').event(exception, 'MY_RULE')
        self.assertEqual(TestAlarmEventProducer.ALARM_ID, event.id)
        self.assertEqual('MY_RULE', event.rule)
        self.assertEqual(exception, event.error)
        self.assertEqual(checker.AlarmEventProducer.FAILED_RESULT, event.result)