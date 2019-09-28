import numpy as np
from PyQt5 import QtCore, QtGui, QtWidgets
from PyQt5.QtCore import Qt
import pyqtgraph as pg
from profiling_data import ProfilingData

pg.setConfigOption('antialias', True)
pg.setConfigOption('background', 'w')
pg.setConfigOption('foreground', 'k')

profiling_dir = "../profiling/2019-09-28_12-28-49/"
pd = ProfilingData(profiling_dir)
deltas = np.subtract(pd.stamps[:, :, :, [1, 3]], pd.stamps[:, :, :, [0, 2]])

class SamplePlot(QtWidgets.QWidget):
    def __init__(self, steps = 5, *args, **kwargs):
        super(SamplePlot, self).__init__(*args, **kwargs)

        layout = QtWidgets.QVBoxLayout()

        self._combo = QtWidgets.QComboBox()
        self._combo.addItems(pd.sample_names)
        layout.addWidget(self._combo)

        self._cpu_plot = pg.PlotWidget(name = 'sample_cpu')
        left_axis = self._cpu_plot.getAxis('left');
        left_axis.enableAutoSIPrefix(enable = False)
        left_axis.setLabel('Time', units = 'ns')
        self._cpu_plot.setLabel('bottom', 'Frame')
        layout.addWidget(self._cpu_plot)

        self._gpu_plot = pg.PlotWidget(name = "sample_gpu")
        left_axis = self._gpu_plot.getAxis('left');
        left_axis.enableAutoSIPrefix(enable = False)
        left_axis.setLabel('Time', units = 'ns')
        self._gpu_plot.setLabel('bottom', 'Frame')
        layout.addWidget(self._gpu_plot)

        self._combo.currentIndexChanged.connect(self.set_sample_index)

        self.setLayout(layout)

        self.set_sample_index(0)

    def set_sample_index(self, sample_index):
        run_index = 0
        self._cpu_plot.clear();
        self._cpu_plot.addItem(pg.PlotDataItem(y = deltas[run_index, :, sample_index, 0]))

        self._gpu_plot.clear();
        self._gpu_plot.addItem(pg.PlotDataItem(y = deltas[run_index, :, sample_index, 1]))

    def set_frame_index(self, frame_index):
        self.frame_index = frame_index

app = QtWidgets.QApplication([])
volume = SamplePlot()
volume.show()
app.exec_()

