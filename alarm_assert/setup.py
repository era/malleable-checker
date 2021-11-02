import os
from setuptools import setup

# Utility function to read the README file.
# Used for the long_description.  It's nice, because now 1) we have a top level
# README file and 2) it's easier to type in the README file than to put a raw
# string in below ...
def read(fname):
    return open(os.path.join(os.path.dirname(__file__), fname)).read()

setup(
    name = "alarm_assert",
    version = "0.0.1",
    author = "Elias Granja",
    author_email = "me@elias.sh",
    description = ("A simple set of functions to assert production data and create alarms"),
    license = "GPLv3",
    packages=['alarm_assert', 'tests'],
    install_requires=[
          'pika',
      ],
    long_description=read('README'),
)