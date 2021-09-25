

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

class CheckerExecutor:
   
    def __init__(self, rule, alarm, datasets):
        self.rule = rule
        self.alarm = alarm
        self.datasets = datasets
    
    def exec(self):
        self.alarm.check(self.rule, self.datasets)
class Alarm:
    
    def alarm(self, exception):
        raise NotImplementedError

    def check(self, rule, datasets):
        try:
            exec(rule, {"datasets": datasets, "FailedAssertion": FailedAssertion})
        except FailedAssertion as e:
            self.alarm(e)