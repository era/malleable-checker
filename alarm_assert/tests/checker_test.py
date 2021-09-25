import unittest

import alarm_assert.checker as checker

class FakeAlarm(checker.Alarm):
    def alarm(self, exception):
        raise exception

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
