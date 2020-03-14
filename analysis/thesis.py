import matplotlib
import matplotlib.pyplot as plt

dpi = 72
textwidth = 448.1309 / dpi

# Using seaborn's style
plt.style.use('seaborn-paper')

matplotlib.rcParams.update({
        "text.usetex": True,
        "font.family": "serif",

        "axes.titlesize": 8,
        "axes.labelsize": 8,
        "font.size": 8,
        "legend.fontsize": 8,
        "xtick.labelsize": 8,
        "ytick.labelsize": 8,

        "lines.linewidth": 0.5,
        "lines.markersize": 0.5,
})
