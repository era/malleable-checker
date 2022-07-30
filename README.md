# Probably not doing anything here anymore
I wanted to play a bit with given the ability for the user to easily write code to use the software I was creating, instead of always using a pre-made user interface. I started this by letting users write python and in the backend I would parse the AST and remove whatever I considered harmful. Also, the code for safety should have been running on a throw-away container.

I decided to stop that and use WASM. So users could write their code in any language that compiles to wasm. Since wasm runs by default in a Sandbox, I could define precisely what the user code had access to.

I played with it a lot, broke a couple of things going forward and made a mess with it. It was fun. But I'm taking what I learnt (the dos and dont's) into (DataBook)[https://codeberg.org/era/databook/] which uses wit (and wit-bindgen) to avoid having to write all the boilerplate code which I played here. So if anything in this project was of interest for you, you should check Databook! Maybe there is something interesting there too (and the code there is more carefully written than this here which mantra was "let me throw some code here to see how this works").

The code here will be left untouched and uncompleted.

# malleable-checker


- A PoC of a [malleable system](https://malleable.systems/) that let's you build rules using a GUI or a script to assert your production data. If something does not pass an assertion, an alarm is triggered.
- This is a toy/pet 🐈‍⬛ project done on my free time, it's going to take a while to be ready for a demonstration.
- There are two versions, one written in Python (check branch python) that is feature complete, although it needs to be polished. The second version is under development and uses WebAssembly + Rust (Host).

# Rust Version (main branch; active development)

WARNING: Not even the README for the Rust/WebAssembly is ready and what is written here is mostly ideas/thoughts/things I need to remember while coding. For a better overview of the project scroll down until you find the Python version explanation.

## About

The main idea of the service is to allow users to write their own scripts to assert assumptions about their database. The user defines datasets (queries that will fetch data from the database), and run assertions on it. If the assertions fail, a message is send to a queue (e.g. rabbitmq).

The Rust version uses WebAssembly to isolate the user script from the system. 

## System design

A webservice written in Rust that executes your .wasm code. The host (written in Rust) executes a wasm module written by the user. The host injects the datasets the user requested together with methods to signal if the host should send an event to a queue (which will potentially trigger an alarm). The module can be written in any language that compiles to wasm.

It exposes the following methods to the wasm env:

### dataset(string)
=> dataset(name) returns the data from the query with named `name`, you can use it to verify your assumptions about the state of your database 

### fail(string)
=> fail(message) should be called when an assertion fails in your code, this will send a failure message to a queue.

### ok(string)
=> ok(message) should be called when an assertion succeeds in your code

You need to expose the following function:

### check()

Check is the function the Rust host will run


## Examples of the wasm code:


## Wasm runtime
We are using https://wasmtime.dev

### How we should use it?

1. Web service (tokio's tower?) accepting wasm and input data, gets a worker from a pool of green threads (Tokio?) and executes the code
2. The response has: status_code, error_description and output (of the wasm script)
3. Wasm can read stdin and write to a stdout. Input from user is given as stdin to wasm env.

## TODO list
- [ ] To think: instead of exposing ok and fail, should we just expose "assertXXX" methods that will operate on the dataset? that would avoid copying the dataset to the wasm env.
- [ ] Webservice exposing HTML and the CRUD (big changes of just keep using the Python one)
- [ ] Rust code to support hosting wasm checkers
- [ ] Examples of checkers in some languages compiled to wasm
- [ ] Run the checkers in a cron-like manner
- [ ] Page with the failed/succeeded checkers (e.g. implement fail and ok functions)
- [ ] Allow users to create namespaces for checkers in order to organise it
- [ ] Allow users to add runbooks to the checkers
- [ ] Allow users to disable alarming from the UI
- [ ] Run the checkers and send an event to a queue in order to alarm


# Python Version (branch: `python`; not supported anymore, still works though)

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
CheckerCase().assertLessThan(datasets['users_with_email_null'], 
  42, 
  'There should be less than 42 users in that state') # Otherwise we are going to alarm
```

The interesting bits is that you can use Python code for it, so you could define your own helpers and write things like:

```python
if bussiness_day():
  CheckerCase().assertLessThan(datasets['users_with_email_null'], 42, 'There should be less than 42 users in that state')
else:
  CheckerCase().assertLessThan(datasets['users_with_email_null'], 5, 'There should be less than 5 users in that state')
```

This is executed hourly by a cron job, if the assertion throws an exception an Alarm will be fired. After the alarm is fired an action is triggered as defined by the user (email, slack, page).

After saved, the alarm has two values red and green. Red means the checker failed and the alarm was triggered. Green means everything is ok.


```mermaid
flowchart TD
    CheckerScheduler -->|every 5 minutes| FetchDataSources
    FetchDataSources --> EvaluateRules
    EvaluateRules --> RunAssertions{Assertion Failed?}
    RunAssertions --> |False| SaveGreenState
    RunAssertions --> |True| TriggerAlarm
    TriggerAlarm --> |Send Message to a queue so user can consume and trigger other actions| MessageQueue
    TriggerAlarm --> SaveRedState
```

## On Images

Basically, you type your SQL query:


![](https://codeberg.org/era/malleable-checker/media/branch/main/create_dataset.png)

Then you can either use the advanced option and write the rule in python or use a normal HTML form to create the rule:

![](https://codeberg.org/era/malleable-checker/media/branch/main/create_checker.png)

### Problems

- Ideally the user defined python code should run in a container, with almost no permission and not able to import anything. Right now the only protection it has is to remove any import AST node. So the user cannot import any module to write/read on disk or to access the sqlite3 database directly.
   - Although they can still run exec/eval in order to do that :P. I need to remove it. //TODO ;)
- Checkers should run on read-only mode, if you have multiple replicas, it should probably read the secondary/read-only.
- Need proper logging.

### What is already done?

- Front-end already works, tho the UX/UI is not done, right now we only have the forms (they are usable)
- Backend already works, you just need to set the cronjob manually

### How to use it?

- To build the dock image just use: `./build_docker.sh`
- To start the docker image just use `./start_docker.sh`
- Frontend is at http://localhost:5000
- RabbitMQ management at `http://localhost:15672` (guest username and password)
- To run the checkers manually, with docker exec open a Bash terminal and:
  - `cd home/checker/` 
  - `CONFIG=config.ini python3 alarm_assert/alarm_assert/checker.py`
- The previous code is also what should go in the cronjob. I didn't commit a crontab file, but you just need to run the previous python script every 5 minutes.


## Structure

The project have two main parts:

- alarm_assert: package defining the DSL for rules and how they should be executed
- front_end: package responsible for the UI and also the front-end database (saving the rules, datasources, rules states and results).
