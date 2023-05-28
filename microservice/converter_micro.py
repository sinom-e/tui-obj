import os
from flask import Flask, request, send_file, render_template
from converter import stl_to_obj, obj_to_stl

app = Flask(__name__)

ALLOWED_EXTENSIONS = {'stl', 'obj'}

def allowed_file(filename):
    return '.' in filename and \
           filename.rsplit('.', 1)[1].lower() in ALLOWED_EXTENSIONS

@app.route('/')
def index():
    return render_template('index.html')

@app.route('/upload', methods=['POST'])
def upload_file():
    file = request.files['file']
    file.save(os.path.join(app.config['UPLOAD_FOLDER'], file.filename))

    if file.filename.endswith('.stl'):
        output_file = stl_to_obj(file.filename)
    elif file.filename.endswith('.obj'):
        output_file = obj_to_stl(file.filename)
    else: 
        return "invalid file"
    get_path = os.path.join('converted', output_file)
    return send_file(get_path, as_attachment=True)

if __name__ == '__main__':
    app.config['UPLOAD_FOLDER'] = 'uploads'
    app.run(debug=True)
