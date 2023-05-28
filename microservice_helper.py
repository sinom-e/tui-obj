import sys
import requests

url = 'http://localhost:5000/upload'

file_path = sys.argv[1] #filename
files = {'file': open(file_path, 'rb')}
response = requests.post(url, files=files)

split_path = file_path.split("/")
new_path = "./microservice/converted/" + split_path[len(split_path) - 1].replace(".stl", ".obj")

print(new_path)
