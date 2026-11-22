import sys

for fn in sys.argv[1:]:
    f = open(fn, 'r')
    data = f.read()
    f.close()
    data = data.replace('\n', '\r\n')
    f = open(fn, 'w', newline='')
    f.write(data)
    f.close()
