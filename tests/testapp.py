from flask import Flask, jsonify, Response
app = Flask(__name__)

@app.route('/auth')
def auth():
    return jsonify({'token': 'abcde'})

@app.route('/things', methods=['GET'])
def things():
    return jsonify({'things':[{"id": "1"}]})

if __name__ == '__main__':
    app.run(debug=True)