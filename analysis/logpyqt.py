import numpy as np
from PyQt5 import QtCore, QtGui, QtWidgets
from PyQt5.QtCore import Qt
import pyqtgraph as pg
from profiling_data import ProfilingData
from PIL import Image

pg.setConfigOption('antialias', True)
pg.setConfigOption('background', 'w')
pg.setConfigOption('foreground', 'k')

profiling_dir = "../profiling/2019-09-28_17-33-11/"


pd = ProfilingData(profiling_dir)
deltas = np.subtract(pd.stamps[:, :, :, [1, 3]], pd.stamps[:, :, :, [0, 2]])

class SamplePlotWidget(QtWidgets.QWidget):
    def __init__(self, cpugpu_index, *args, **kwargs):
        super(SamplePlotWidget, self).__init__(*args, **kwargs)

        self.sample_index = 0
        self.cpugpu_index = cpugpu_index

        self._sample_combo = QtWidgets.QComboBox()
        self._sample_combo.addItems(pd.sample_names)
        self._sample_combo.currentIndexChanged.connect(self.set_sample_index)

        self._cpugpu_combo = QtWidgets.QComboBox()
        self._cpugpu_combo.addItems(['CPU', 'GPU'])
        self._cpugpu_combo.currentIndexChanged.connect(self.set_cpugpu_index)

        self._plot = pg.PlotWidget(name = 'sample_cpu')
        left_axis = self._plot.getAxis('left');
        left_axis.enableAutoSIPrefix(enable = False)
        left_axis.setLabel('Time', units = 'ns')
        self._plot.setLabel('bottom', 'Frame')

        h_layout = QtWidgets.QHBoxLayout()
        h_layout.addWidget(self._sample_combo)
        h_layout.addWidget(self._cpugpu_combo)

        v_layout = QtWidgets.QVBoxLayout()
        v_layout.addLayout(h_layout)
        v_layout.addWidget(self._plot)
        self.setLayout(v_layout)

        self.update_plot()

    def set_sample_index(self, sample_index):
        self.sample_index = sample_index
        self.update_plot()

    def set_cpugpu_index(self, cpugpu_index):
        self.cpugpu_index = cpugpu_index
        self.update_plot()

    def update_plot(self):
        self._plot.clear();
        self._plot.addItem(pg.PlotDataItem(y = deltas[0, :, self.sample_index, self.cpugpu_index]))

    def set_frame_index(self, frame_index):
        self.frame_index = frame_index

class MainWidget(QtWidgets.QWidget):
    def __init__(self, *args, **kwargs):
        super(MainWidget, self).__init__(*args, **kwargs)

        layout = QtWidgets.QGridLayout()

        self._cpu = SamplePlotWidget(0)
        self._gpu = SamplePlotWidget(1)
        self._frame = pg.ImageView()
        self._hist = SamplePlotWidget(1)

        self._frame_slider = QtWidgets.QSlider(Qt.Horizontal)
        self._frame_slider.setMinimum(0)
        self._frame_slider.setMaximum(pd.frame_count)
        self._frame_slider.valueChanged.connect(self.set_frame_index)

        layout.addWidget(self._cpu, 0, 0)
        layout.addWidget(self._gpu, 1, 0)
        layout.addWidget(self._frame, 0, 1)
        layout.addWidget(self._hist, 1, 1)
        layout.addWidget(self._frame_slider, 2, 0, 1, 2)

        self.setLayout(layout)

    def set_frame_index(self, frame_index):
        self._cpu.set_frame_index(frame_index)
        self._gpu.set_frame_index(frame_index)

        im = Image.open("{}frames/{}.bmp".format(profiling_dir, frame_index))
        im = np.array(im)
        print(np.shape(im))

        self._frame.setImage(im)

app = QtWidgets.QApplication([])
main = MainWidget()
main.show()
app.exec_()

