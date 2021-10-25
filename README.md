# malleable-checker

- A PoC of a [malleable system](https://malleable.systems/) that let's you build rules using a GUI or a script to assert your production data. If something does not pass an assertion, an alarm is triggered.
- This is a toy/pet 🐈‍⬛ project done on my free time, it's going to take a while to be ready for a demonstration.


## How it works?

- You define the datasource (e.g. an SQL query)
- Using either a HTML form or a python script you assert what you expect of that data.
- If the assertions fail, the alarm is triggered

For example, with the following datasource:
```sql
// Data source id = 32, name = 'users_with_email_null'
SELECT username from users where email is null;
```

You can create a new rule:

```python
CheckerCase().assertLessThan(datasets['users_with_email_null'], 42, 'There should be less than 42 users in that state') # Otherwise we are going to alarm

```

This is executed hourly by a cron job, if the assertion throws an exception an Alarm will be fired. After the alarm is fired an action is triggered as defined by the user (email, slack, page).

After saved, the alarm has two values red and green. Red means the checker failed and the alarm was triggered. Green means everything is ok.

![](overview.png)

### Problems

- Ideally the user defined python code should run in a container, with almost no permission and not able to import anything. Right now, I don't check anything, so it is not safe as it is.
  - This could be implemented by getting the ast (https://docs.python.org/3/library/ast.html) and removing things like import from it.

## On Images

Basically, you type your SQL query:


<img width="625" alt="datasource" src="https://user-images.githubusercontent.com/266034/134243490-6a8a14e6-5c68-4746-a564-47552d9ae1d9.png">

Then you can either use the advanced option and write the rule in python or use a normal HTML form to create the rule:

<img width="568" alt="rule" src="https://user-images.githubusercontent.com/266034/134243621-1ce23431-cd62-4c0a-9f24-2acf3013a9e6.png">


## Structure

The project have two main parts:

- alarm_assert: package defining the DSL for rules and how they should be executed
- front_end: package responsible for the UI and also the front-end database (saving the rules, datasources, rules states and results).
