import unittest

import alarm_assert.checker as checker

class FakeAlarm(checker.Alarm):
    def alarm(self, exception):
        raise exception

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
        
