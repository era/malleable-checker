

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
    pass

class Alarm:
    
    def alarm(self, exception):
        raise NotImplementedError

    def check(self, function):
        try:
            function()
        except FailedAssertion as e:
            self.alarm(e)