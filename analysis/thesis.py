import matplotlib
import matplotlib.pyplot as plt
import numpy as np

golden_ratio = (1 + np.sqrt(5))/2
inches_per_pt = 1/72
textwidth_pt = 448.1309 # pt
textwidth_inch = textwidth_pt * inches_per_pt

fig_width = textwidth_inch
fig_dims = (fig_width, fig_width/golden_ratio)

# Using seaborn's style
plt.style.use('seaborn-paper')

matplotlib.rcParams.update({
        # Use LaTeX to write all text
        "text.usetex": True,
        "font.family": "serif",
        # Use 10pt font in plots, to match 10pt font in document
        "axes.labelsize": 10,
        "font.size": 10,
        # Make the legend/label fonts a little smaller
        "legend.fontsize": 8,
        "xtick.labelsize": 8,
        "ytick.labelsize": 8,
})
