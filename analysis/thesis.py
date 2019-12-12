import matplotlib
import matplotlib.pyplot as plt

dpi = 72
textwidth = 448.1309 / dpi

# Using seaborn's style
plt.style.use('seaborn-paper')

matplotlib.rcParams.update({
        # Use LaTeX to write all text
        "text.usetex": True,
        "font.family": "serif",
        # Use 10pt font in plots, to match 10pt font in document
        "axes.labelsize": 8,
        "font.size": 8,
        # Make the legend/label fonts a little smaller
        "legend.fontsize": 8,
        "xtick.labelsize": 8,
        "ytick.labelsize": 8,
})
