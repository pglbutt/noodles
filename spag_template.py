"""
Implements a simple template for parameterizing our requests.

Basic rules:

    - Use double braces to define a list: {{item1, item2, ...}}
    - Attempt to produce a value by evaluating each item in the list, in order.
    - The entire text "{{...}}" is replaced with the first item that
    successfully yields a value.
    - Items are evaulated as follows:
        - If the item does not contain a '.' then the value is specified using
        the `--with` option:
            e.g. "{{thing_id}}" and "--with thing_id=poo" evaluates to "poo"
        - If the item contains a period, then the value is found in a
        remembered request file
            e.g. "{{last.response.body.id}}" says to look in a file last.yml
            and find the value keyed by response.body.id

Shortcut rules:

    - Double exclamation point:
        - If <expr> does not contain a '.', then @<expr> expands to
        {{last.response.body.<expr>}}.
            /thing/@id --> /thing/{{last.response.body.id}}
        - If <expr> contains a '.', then "@<expr>" expands to
        {{last.response.<expr>}}.
            /thing/@headers.id --> /thing/{{last.response.headers.id}}
"""
import json
import string

import common
import spag_remembers
import spag_files


def _find_double_braces(s):
    """Return (a, b) such that s[a:b] returns the first "{{...}}" in s.
    Return None on failure. Raise exception on unclosed double braces.
    """
    i = s.find("{{")
    if i < 0:
        return None
    j = s.find("}}", i)
    if j < 0:
        raise common.ToughNoodles("Unclosed braces: '{0}'".format(s[i:i+20]))
    return i, j + 2

def _substitute_braces(s, withs, body_type):
    """Replace a string "{{...}}" with a value. Raises ToughNoodles on bad
    formatting or failing to substitute.

    :param s: A string with a double-brace list: "{{...}}"
    :param withs: A dictionary containing the --with options
    :param body_type: If 'json', treat response.body as json
    """
    original = s
    assert s[:2] == "{{" and s[-2:] == "}}"
    s = s[2:-2]

    # don't allow any braces within double-brace sections
    # note: {{ is the escape sequence for a single { in a python string
    if '{' in s or '}' in s:
        raise common.ToughNoodles("'{{' and '}}' not allowed in double-braced "
                                  "list: '{0}'".format(original))
    if '@' in s:
        raise common.ToughNoodles("Shortcut '@' not allowed in double-braced "
                                  "list: '{0}'".format(original))

    items = [x.strip() for x in s.split(',') if x.strip()]

    for item in items:
        # items without a dot are specfied using `--with item=val`
        if '.' not in item and item in withs:
            return withs[item]
        elif '.' not in item:
            continue

        try:
            return _lookup_item_with_dot(item, body_type)
        except common.ToughNoodles:
            pass

    raise common.ToughNoodles("Failed to substitute for {0}".format(original))

def _lookup_item_with_dot(item, body_type):
    # for "thing.something.maybe.more", find thing.yml in the remembered files
    parts = item.split('.')
    name, lookup = parts[0], parts[1:]

    path = spag_remembers.SpagRemembers().get_path(name)
    data = spag_files.load_file(path)

    # load the body as json
    if body_type == 'json':
        try:
            body = data['response']['body']
            if 'body' in data['response'] and type(body) is str:
                data['response']['body'] = json.loads(body)
        except ValueError as e:
            pass
            # print("WARNING: {0} on item {1} with file {2}".format(str(e), item, path))
            # failed to load json, but keep trying. maybe the user
            # doesn't need the json body.

    # if lookup = ['a', 'b', 'c'] then we need data['a']['b']['c']
    # print >>sys.stderr, data
    try:
        for key in lookup:
            if key in data:
                data = data[key]
            else:
                raise common.ToughNoodles
        return data
    except common.ToughNoodles:
        pass
    except TypeError:
        # raised if we tried LIST['poo'] or STRING['poo']
        pass
    except IndexError:
        # raised if we tried LIST[999999999] or STRING[99999999]
        pass

    raise common.ToughNoodles

VALID_BANG_CHARS = string.ascii_letters + string.digits + "_."
def _substitute_bangs(s, body_type):
    assert s.startswith('@')
    original = s
    s = s[len('@'):]
    if not s:
        raise common.ToughNoodles("No key found after @")
    if s[0] not in VALID_BANG_CHARS:
        raise common.ToughNoodles("Invalid char '{0}' found immediately after @"
                           .format(s[0]))

    count = 0
    for c in s:
        if c not in VALID_BANG_CHARS:
            break
        count += 1

    # in case we have '@....poo', just compress the dots into a single dot
    key = s[:count].strip('.')

    if '.' in key:
        key = 'last.response.' + key
    else:
        key = 'last.response.body.' + key

    return _lookup_item_with_dot(key, body_type) + s[count:]


def untemplate(s, withs={}, body_type='json', shortcuts=False):
    """Process the {{ }} sections of the template strings

    :param shortcuts: Look for shortcut syntax '@id' '@last.id' '@body.id'
    """
    withs = parse_withs(withs)
    while True:
        bounds = _find_double_braces(s)
        if not bounds:
            break
        a, b = bounds
        s = s[:a] + _substitute_braces(s[a:b], withs, body_type) + s[b:]

    if shortcuts:
        while True:
            i = s.find('@')
            if i < 0:
                break
            s = s[:i] + _substitute_bangs(s[i:], body_type)
    return s

def split_with(w):
    parts = w.split('=', 1)
    if len(parts) < 2:
        raise common.ToughNoodles("Bad with argument {0}".format(w))
    return parts

def parse_withs(withs):
    return {k: v for k, v in (split_with(w) for w in withs)}
