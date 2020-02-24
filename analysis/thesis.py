import matplotlib
import matplotlib.pyplot as plt

dpi = 72
figsize = (1600/dpi, 900/dpi)

# Using seaborn's style
plt.style.use('seaborn-paper')

matplotlib.rcParams.update({
        "text.usetex": True,
        "font.family": "serif",

        "axes.titlesize": 24,
        "axes.labelsize": 24,
        "font.size": 24,
        "legend.fontsize": 24,
        "xtick.labelsize": 24,
        "ytick.labelsize": 24,

        "lines.linewidth": 3,
        "lines.markersize": 3,
})
