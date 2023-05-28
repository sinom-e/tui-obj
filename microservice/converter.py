import sys
import os.path
import numpy as np
import meshio
from stl import mesh
import trimesh
import shutil

def stl_to_obj(file):
    stl_file = os.path.join('uploads', file)
    stl_mesh = mesh.Mesh.from_file(stl_file)

    vertices = stl_mesh.vectors.reshape((-1, 3)) # Extract the vertices and faces from the STL mesh
    cells = [('triangle', np.arange(len(vertices)).reshape((-1, 3)))]

    meshio_mesh = meshio.Mesh( # Create a meshio Mesh object from the vertices and faces
        points=vertices,
        cells=cells
    )

    output_file = os.path.splitext(file)[0] + '.obj' # Write the Mesh object to an OBJ file
    meshio.write(output_file, meshio_mesh, file_format='obj')

    dest_file = os.path.join('converted', os.path.basename(output_file)) #move the converted file into converted directory
    shutil.move(output_file, dest_file)

    return output_file

def obj_to_stl(file):
    stl_file = os.path.join('uploads', file)
    mesh = trimesh.load(stl_file)

    output_file = os.path.splitext(file)[0] + '.stl' # Write the mesh to an STL file
    mesh.export(output_file)

    dest_file = os.path.join('converted', os.path.basename(output_file)) #move the converted file into converted directory
    shutil.move(output_file, dest_file)

    return output_file

