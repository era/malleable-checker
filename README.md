# malleable-checker

- A PoC of a malleable system that let's you build rules using a GUI or a script to assert your production data. If something does not pass an assertion, an alarm is triggered.
- This is a toy project done on my free time, it's going to take a while to be ready for a demonstration.


## How it works?

- On the UI using either a HTML form or a python script you define the datasource of your alarm (e.g. an SQL query) and you assert what you expect of that data.

For example, with the following datasource:
```sql
// Data source id = 32
SELECT username from users where email is null;
```

You can create a new rule:

```python
datasource = DataSource.find(id=32) # Query looking for users without an email
checker = CheckerCase()

checker.assertLessThan(datasource, 42, 'There should be less than 42 users in that state') # Otherwise we are going to alarm

```

This is executed hourly by a cron job, if the assertion throws an exception an Alarm will be fired. The alarm can be defined by the user (email, slack, page).

- The datasource query / API call / any other thing must be saved in the database prior to making the checker rule. There will be an interface for defining those as well.

- Ideally the user defined python code should run in a container, with almost no permission and not able to import anything. Right now, I don't check anything, so it is not safe as it is.


## Structure

The project have two main parts:
- alarm_assert: package defining the DSL for rules and how they should be executed
- front_end: package responsible for the UI and also the front-end database (saving the rules, datasources, rules states and results).