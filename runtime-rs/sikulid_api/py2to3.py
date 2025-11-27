# -*- coding: utf-8 -*-
"""
Python 2 to Python 3 Converter / Python 2 から Python 3 への変換器

Converts Python 2 code to Python 3 using regex-based transformations.
正規表現ベースの変換でPython 2コードをPython 3に変換します。

Note: lib2to3 was removed in Python 3.13, so this uses a custom implementation.
注意: Python 3.13でlib2to3が削除されたため、カスタム実装を使用します。

This module can be used:
1. As a library - import and call convert_code()
2. As a command line tool - python py2to3.py <input> [output]

Usage / 使用方法:
    # Convert file / ファイルを変換
    python py2to3.py input.py output.py

    # Convert from stdin to stdout / 標準入出力で変換
    python py2to3.py - -

    # Convert in-place / その場で変換
    python py2to3.py input.py
"""
from __future__ import print_function

import sys
import os
import re


class Py2To3Converter:
    """
    Python 2 to Python 3 converter using regex transformations.
    正規表現変換を使用したPython 2からPython 3への変換器。

    Handles common Python 2 constructs:
    Python 2の一般的な構文を処理:
    - print statement → print() function
    - xrange() → range()
    - raw_input() → input()
    - except Exception, e: → except Exception as e:
    - Long literals (123L) → int literals
    - basestring → str
    - execfile() → exec(open().read())
    - has_key() → in operator
    - unicode() → str()
    - And more...
    """

    def __init__(self):
        """Initialize converter with transformation rules."""
        # Compile regex patterns for performance
        self._patterns = self._compile_patterns()

    def _compile_patterns(self):
        """Compile all transformation patterns."""
        return [
            # print statement to print function
            # Handle print with no arguments: print → print()
            (
                re.compile(r'^(\s*)print\s*$', re.MULTILINE),
                r'\1print()'
            ),
            # Handle print with arguments: print "text" → print("text")
            (
                re.compile(r'^(\s*)print\s+(?!\()(.+)$', re.MULTILINE),
                self._convert_print
            ),
            # xrange → range
            (re.compile(r'\bxrange\s*\('), r'range('),
            # raw_input → input
            (re.compile(r'\braw_input\s*\('), r'input('),
            # except Exception, e: → except Exception as e:
            (
                re.compile(r'except\s+(\w+(?:\.\w+)*)\s*,\s*(\w+)\s*:'),
                r'except \1 as \2:'
            ),
            # Long literals: 123L → 123
            (re.compile(r'\b(\d+)[Ll]\b'), r'\1'),
            # basestring → str
            (re.compile(r'\bbasestring\b'), r'str'),
            # unicode → str (as function call)
            (re.compile(r'\bunicode\s*\('), r'str('),
            # <> → !=
            (re.compile(r'\s*<>\s*'), r' != '),
            # has_key() → in operator
            (
                re.compile(r'(\w+)\.has_key\s*\(\s*([^)]+)\s*\)'),
                r'(\2 in \1)'
            ),
            # dict.iteritems() → dict.items()
            (re.compile(r'\.iteritems\s*\(\s*\)'), r'.items()'),
            # dict.iterkeys() → dict.keys()
            (re.compile(r'\.iterkeys\s*\(\s*\)'), r'.keys()'),
            # dict.itervalues() → dict.values()
            (re.compile(r'\.itervalues\s*\(\s*\)'), r'.values()'),
            # execfile('file') → exec(open('file').read())
            (
                re.compile(r'\bexecfile\s*\(\s*([^)]+)\s*\)'),
                r'exec(open(\1).read())'
            ),
            # reduce import
            # functools.reduce is needed in Python 3
            # This adds import at top if reduce is used
        ]

    def _convert_print(self, match):
        """
        Convert print statement to print function.
        print文をprint関数に変換。
        """
        indent = match.group(1)
        content = match.group(2).strip()

        # Handle trailing comma (no newline)
        trailing_comma = content.endswith(',')
        if trailing_comma:
            content = content[:-1].strip()

        # Handle >> redirect: print >> file, args
        redirect_match = re.match(r'>>\s*(\w+)\s*,\s*(.+)', content)
        if redirect_match:
            file_obj = redirect_match.group(1)
            args = redirect_match.group(2)
            if trailing_comma:
                return '{}print({}, file={}, end=" ")'.format(indent, args, file_obj)
            else:
                return '{}print({}, file={})'.format(indent, args, file_obj)

        # Handle multiple items separated by comma
        if trailing_comma:
            return '{}print({}, end=" ")'.format(indent, content)
        else:
            return '{}print({})'.format(indent, content)

    def convert_code(self, source):
        """
        Convert Python 2 code to Python 3.
        Python 2コードをPython 3に変換。

        Args:
            source: Python 2 source code string
                    Python 2ソースコード文字列

        Returns:
            Converted Python 3 source code string
            変換されたPython 3ソースコード文字列
        """
        result = source

        # Check if 'from __future__ import print_function' is present
        has_print_function_import = 'from __future__ import print_function' in source

        for pattern, replacement in self._patterns:
            if callable(replacement):
                result = pattern.sub(replacement, result)
            else:
                result = pattern.sub(replacement, result)

        # Remove __future__ imports (no longer needed in Python 3)
        result = re.sub(
            r'^from __future__ import [^\n]+\n',
            '',
            result,
            flags=re.MULTILINE
        )

        # Handle encoding declaration (not needed in Python 3 but harmless)
        # Leave encoding comments alone as they're still valid

        return result

    def convert_file(self, input_path, output_path=None):
        """
        Convert a Python 2 file to Python 3.
        Python 2ファイルをPython 3に変換。

        Args:
            input_path: Path to input Python 2 file
                        入力Python 2ファイルのパス
            output_path: Path to output Python 3 file. If None, modifies in-place.
                        出力Python 3ファイルのパス。Noneの場合、その場で修正。

        Returns:
            Converted source code
            変換されたソースコード
        """
        with open(input_path, 'r', encoding='utf-8') as f:
            source = f.read()

        converted = self.convert_code(source)

        if output_path:
            with open(output_path, 'w', encoding='utf-8') as f:
                f.write(converted)
        elif output_path is None:
            # In-place modification
            with open(input_path, 'w', encoding='utf-8') as f:
                f.write(converted)

        return converted


# Global converter instance for convenience
_converter = None


def get_converter():
    """Get or create the global converter instance."""
    global _converter
    if _converter is None:
        _converter = Py2To3Converter()
    return _converter


def convert_code(source):
    """
    Convert Python 2 code to Python 3.
    Python 2コードをPython 3に変換する便利関数。

    Args:
        source: Python 2 source code string
                Python 2ソースコード文字列

    Returns:
        Converted Python 3 source code string
        変換されたPython 3ソースコード文字列
    """
    return get_converter().convert_code(source)


def convert_file(input_path, output_path=None):
    """
    Convert a Python 2 file to Python 3.
    Python 2ファイルをPython 3に変換する便利関数。

    Args:
        input_path: Path to input Python 2 file
                    入力Python 2ファイルのパス
        output_path: Path to output Python 3 file. If None, modifies in-place.
                    出力Python 3ファイルのパス。Noneの場合、その場で修正。

    Returns:
        Converted source code
        変換されたソースコード
    """
    return get_converter().convert_file(input_path, output_path)


def main():
    """Command line interface for py2to3 converter."""
    import argparse

    parser = argparse.ArgumentParser(
        description='Convert Python 2 code to Python 3'
    )
    parser.add_argument(
        'input',
        help='Input file path or "-" for stdin'
    )
    parser.add_argument(
        'output',
        nargs='?',
        help='Output file path or "-" for stdout. If omitted, modifies in-place.'
    )
    parser.add_argument(
        '-v', '--verbose',
        action='store_true',
        help='Verbose output'
    )

    args = parser.parse_args()

    try:
        if args.input == '-':
            # Read from stdin
            source = sys.stdin.read()
            converted = convert_code(source)

            if args.output == '-' or args.output is None:
                print(converted, end='')
            else:
                with open(args.output, 'w', encoding='utf-8') as f:
                    f.write(converted)
        else:
            # Read from file
            if not os.path.exists(args.input):
                print("Error: Input file not found: {}".format(args.input), file=sys.stderr)
                sys.exit(1)

            if args.output == '-':
                # Output to stdout
                with open(args.input, 'r', encoding='utf-8') as f:
                    source = f.read()
                converted = convert_code(source)
                print(converted, end='')
            else:
                # Output to file
                converted = convert_file(args.input, args.output)

            if args.verbose:
                print("Converted: {} -> {}".format(
                    args.input,
                    args.output or args.input
                ), file=sys.stderr)

    except Exception as e:
        print("Error: {}".format(e), file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
