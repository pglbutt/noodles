from flask import Flask, jsonify, request
app = Flask(__name__)

database = set()


@app.route('/auth')
def auth():
    return jsonify({'token': 'abcde'})


@app.route('/things', methods=['GET'])
def list_things():
    things = [{"id": x} for x in database]
    return jsonify({'things': things})


@app.route('/things', methods=['POST'])
def post_thing():
    # doesn't work unless content-type: application/json
    if request.json is None:
        print("probably need application/json")
        return ('', 400)
    thing_id = request.json.get('id')
    if thing_id is None:
        return ('', 400)
    elif thing_id in database:
        return ('', 409)
    database.add(str(thing_id))
    return jsonify({"id": str(thing_id)}), 201


@app.route('/things/<id>', methods=['GET'])
def get_thing(id):
    if id not in database:
        return ('', 404)
    return jsonify({"id": id})


@app.route('/things/<id>', methods=['PUT', 'PATCH'])
def modify_thing(id):
    if id not in database:
        return ('', 404)
    else:
        thing_id = request.json.get('id')
        database.remove(id)
        database.add(thing_id)
    return jsonify({"id": str(thing_id)}), 201


@app.route('/things/<id>', methods=['DELETE'])
def delete_thing(id):
    if id in database:
        database.remove(id)
    return ('', 204)


@app.route('/clear', methods=['GET', 'POST', 'DELETE'])
def clear():
    database.clear()
    return ('', 204)


@app.route('/headers', methods=['GET'])
def headers():
    ignored_headers = ['Content-Length',
                       'Accept-Encoding',
                       'Host',
                       'Accept',
                       'User-Agent',
                       'Connection',
                       'Content-Type']
    return jsonify({key: value for key, value in
                    request.headers.items() if key not in ignored_headers})


@app.route('/params', methods=['GET'])
def params():
    return jsonify({key: value for key, value in request.args.items()})

if __name__ == '__main__':
    app.run(debug=True)
