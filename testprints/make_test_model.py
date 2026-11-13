"""Write the STL models for the Embercast test prints.

The plate model is a base plate of 30 x 20 x 1.5 mm with three parts on top:
a 10 mm cube, the letter F and three thin square pins. The script writes the
model as boxes. The boxes overlap, and the slicer fills the union of the
boxes. The cube model is one solid 20 mm cube for the hollowing test. Run the
script with the output path, and optionally "cube" as the second argument.
"""

import struct
import sys

CUBE = [((0.0, 0.0, 0.0), (20.0, 20.0, 20.0))]

BOXES = [
    ((0.0, 0.0, 0.0), (30.0, 20.0, 1.5)),
    ((2.0, 4.0, 1.5), (12.0, 14.0, 11.5)),
    ((15.0, 3.0, 1.5), (17.0, 17.0, 4.5)),
    ((15.0, 15.0, 1.5), (24.0, 17.0, 4.5)),
    ((15.0, 9.0, 1.5), (22.0, 11.0, 4.5)),
    ((27.0, 15.5, 1.5), (28.0, 16.5, 4.5)),
    ((27.2, 9.7, 1.5), (27.8, 10.3, 4.5)),
    ((27.3, 3.8, 1.5), (27.7, 4.2, 4.5)),
]

CORNER_FACES = [
    (0, 2, 1), (0, 3, 2),
    (4, 5, 6), (4, 6, 7),
    (0, 1, 5), (0, 5, 4),
    (1, 2, 6), (1, 6, 5),
    (2, 3, 7), (2, 7, 6),
    (3, 0, 4), (3, 4, 7),
]


def box_triangles(low, high):
    """Return the 12 outward facing triangles of the box between low and high."""
    (x0, y0, z0), (x1, y1, z1) = low, high
    corners = [
        (x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0),
        (x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1),
    ]
    return [tuple(corners[i] for i in face) for face in CORNER_FACES]


def normal(a, b, c):
    """Return the unit normal of the triangle a, b, c."""
    u = [b[i] - a[i] for i in range(3)]
    v = [c[i] - a[i] for i in range(3)]
    n = (u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0])
    length = sum(x * x for x in n) ** 0.5
    return tuple(x / length for x in n)


def write_stl(path, boxes):
    """Write all boxes to path as one binary STL file."""
    triangles = [t for low, high in boxes for t in box_triangles(low, high)]
    with open(path, "wb") as stl:
        stl.write(b"Embercast test print 1".ljust(80, b" "))
        stl.write(struct.pack("<I", len(triangles)))
        for a, b, c in triangles:
            stl.write(struct.pack("<3f", *normal(a, b, c)))
            for vertex in (a, b, c):
                stl.write(struct.pack("<3f", *vertex))
            stl.write(struct.pack("<H", 0))


if __name__ == "__main__":
    model = sys.argv[2] if len(sys.argv) > 2 else "plate"
    write_stl(sys.argv[1], CUBE if model == "cube" else BOXES)
