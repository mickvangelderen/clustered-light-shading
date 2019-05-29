import matplotlib.pyplot as plt;
import numpy as np;
import sympy as sp;

c0 = 1.0
c2 = 0.42
radius = 15.0

def compute_c1():
    c1 = sp.symbols('c1')
    x = radius
    return sp.solve (1./(c0 + c1*x + c2*x*x), c1)

c1 = compute_c1()

print(c1)

x = np.arange(0., radius, 0.1)
y = 1./(c0 + c1*x + c2*x*x)

print(y[129])

plt.plot(x, y)
plt.show()
