from flask import Flask, jsonify, request
app = Flask(__name__)

@app.route('/auth')
def auth():
    return jsonify({'token': 'abcde'})

@app.route('/things', methods=['GET'])
def things():
    return jsonify({'things':[{"id": "1"}]})

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

if __name__ == '__main__':
    app.run(debug=True)