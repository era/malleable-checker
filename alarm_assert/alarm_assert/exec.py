import fileinput
import re
import sys
from alarm_assert.checker import CheckerCase


def main(args):
    input_lines = fileinput.input(args)
    code = "".join(input_lines)
    exec(code)


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
