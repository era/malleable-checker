from setuptools import setup

setup(name='front_end',
      version='0.0.1',
      #url='http://github.com/era/mallable-checker',
      author='Elias Granja',
      author_email='me@elias.sh',
      license='GPLv3',
      packages=['server'],
      install_requires=[
          'flask',
      ],
      zip_safe=False)
