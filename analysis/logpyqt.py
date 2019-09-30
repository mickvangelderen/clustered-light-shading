import numpy as np
from PyQt5 import QtCore, QtGui, QtWidgets
from PyQt5.QtCore import Qt
import pyqtgraph as pg
from profiling_data import ProfilingData
from PIL import Image

pg.setConfigOption('antialias', True)
pg.setConfigOption('background', 'w')
pg.setConfigOption('foreground', 'k')

profiling_dir = "../profiling/2019-09-30_17-05-39/"

pd = ProfilingData(profiling_dir)
deltas = np.subtract(pd.stamps[:, :, :, [1, 3]], pd.stamps[:, :, :, [0, 2]])

class SamplePlotWidget(QtWidgets.QWidget):
    def __init__(self, cpugpu_index, *args, **kwargs):
        super(SamplePlotWidget, self).__init__(*args, **kwargs)

        self.sample_index = 0
        self.frame_index = 0
        self.cpugpu_index = cpugpu_index

        self._sample_combo = QtWidgets.QComboBox()
        self._sample_combo.addItems(pd.sample_names)
        self._sample_combo.currentIndexChanged.connect(self.set_sample_index)

        self._cpugpu_combo = QtWidgets.QComboBox()
        self._cpugpu_combo.addItems(['CPU', 'GPU'])
        self._cpugpu_combo.setCurrentIndex(cpugpu_index)
        self._cpugpu_combo.currentIndexChanged.connect(self.set_cpugpu_index)

        self._plot = pg.PlotWidget(name = 'sample_cpu')
        left_axis = self._plot.getAxis('left');
        left_axis.enableAutoSIPrefix(enable = False)
        left_axis.setLabel('Time', units = 'ns')
        self._plot.setLabel('bottom', 'Frame')

        self._plot_runs = [pg.PlotDataItem() for _ in range(0, pd.run_count)]
        for pdi in self._plot_runs:
            self._plot.addItem(pdi)
        for run_index in range(1, pd.run_count):
            self._plot_runs[run_index].setPen(color = QtGui.QColor(127, 127, 127, 255/pd.run_count))
        self._plot_runs_med = pg.PlotDataItem()
        self._plot.addItem(self._plot_runs_med)
        self._plot_runs_med.setPen(color = QtGui.QColor(100, 100, 255, 255))

        self._plot_frame_line = pg.InfiniteLine(self.frame_index, bounds = [0, pd.frame_count - 1]);
        self._plot.addItem(self._plot_frame_line)

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
        for run_index in range(1, pd.run_count):
            self._plot_runs[run_index].setData(y = deltas[run_index, :, self.sample_index, self.cpugpu_index])
        y = np.min(deltas[1:, :, self.sample_index, self.cpugpu_index], axis = 0, keepdims = True)
        self._plot_runs_med.setData(y = y[0, :])

    def set_frame_index(self, frame_index):
        self.frame_index = frame_index
        self._plot_frame_line.setPos(self.frame_index)

class HistPlotWidget(QtWidgets.QWidget):
    def __init__(self, hist_index, *args, **kwargs):
        super(HistPlotWidget, self).__init__(*args, **kwargs)

        # self.sample_index = 0
        self.frame_index = 0
        self.hist_index = hist_index

        self._sample_combo = QtWidgets.QComboBox()
        # self._sample_combo.addItems(pd.sample_names)
        # self._sample_combo.currentIndexChanged.connect(self.set_sample_index)

        self._hist_combo = QtWidgets.QComboBox()
        self._hist_combo.addItems(['Fragments per cluster', 'Light indices per cluster', 'Light indices per fragment'])
        self._hist_combo.currentIndexChanged.connect(self.set_hist_index)

        self._plot = pg.PlotWidget(name = 'sample_cpu')
        left_axis = self._plot.getAxis('left');
        left_axis.enableAutoSIPrefix(enable = False)

        g_layout = QtWidgets.QGridLayout()
        self._cluster_count_label = QtWidgets.QLabel()
        self._light_indices_count_label = QtWidgets.QLabel()
        self._fragment_count_label = QtWidgets.QLabel()
        g_layout.addWidget(QtWidgets.QLabel("visible cluster count"), 0, 0)
        g_layout.addWidget(self._cluster_count_label, 0, 1)
        g_layout.addWidget(QtWidgets.QLabel("total light indices"), 1, 0)
        g_layout.addWidget(self._light_indices_count_label, 1, 1)
        g_layout.addWidget(QtWidgets.QLabel("total fragments in cluster space"), 2, 0)
        g_layout.addWidget(self._fragment_count_label, 2, 1)

        h_layout = QtWidgets.QHBoxLayout()
        h_layout.addWidget(self._sample_combo)
        h_layout.addWidget(self._hist_combo)

        v_layout = QtWidgets.QVBoxLayout()
        v_layout.addLayout(h_layout)
        v_layout.addLayout(g_layout)
        v_layout.addWidget(self._plot)
        self.setLayout(v_layout)

        self.update_plot()

    def set_hist_index(self, hist_index):
        self.hist_index = hist_index
        self.update_plot()

    def update_plot(self):
        self._plot.clear();

        if self.hist_index == 0:
            data = pd.cluster_buffers[:, 0, 5:36]
            x = np.arange(1, 33, dtype = np.uint64)
            y = data[self.frame_index, :]
            self._plot.setYRange(np.min(data), np.max(data))
            self._plot.setLabel('left', 'Cluster Count')
            self._plot.setLabel('bottom', 'log_2(Fragments)')

        elif self.hist_index == 1:
            data = pd.cluster_buffers[:, 0, 36:68]
            x = 8*np.arange(-0.5, 32.5)
            y = data[self.frame_index, :]
            self._plot.setYRange(np.min(data), np.max(data))
            self._plot.setLabel('left', 'Cluster Count')
            self._plot.setLabel('bottom', 'Light Count')

        elif self.hist_index == 2:
            data = pd.cluster_buffers[:, 0, 68:100]
            x = 8*np.arange(-0.5, 32.5)
            y = data[self.frame_index, :]
            self._plot.setYRange(np.min(data), np.max(data))
            self._plot.setLabel('left', 'Fragment Count')
            self._plot.setLabel('bottom', 'Light Count')

        self._plot.addItem(pg.PlotDataItem(x, y, stepMode = True))

        self._cluster_count_label.setText("{}".format(pd.cluster_buffers[self.frame_index, 0, 0]))
        self._light_indices_count_label.setText("{}".format(pd.cluster_buffers[self.frame_index, 0, 1]))
        self._fragment_count_label.setText("{}".format(np.sum(pd.cluster_buffers[self.frame_index, 0, 68:100])))

    def set_frame_index(self, frame_index):
        self.frame_index = frame_index
        self.update_plot()

class MainWidget(QtWidgets.QWidget):
    def __init__(self, *args, **kwargs):
        super(MainWidget, self).__init__(*args, **kwargs)

        layout = QtWidgets.QGridLayout()

        self._cpu = SamplePlotWidget(0)
        self._gpu = SamplePlotWidget(1)
        self._frame_img = pg.ImageItem()
        self._frame_widget = pg.GraphicsLayoutWidget()
        self._frame_vb = self._frame_widget.addViewBox(row = 1, col = 1)
        self._frame_vb.setAspectLocked()
        self._frame_vb.addItem(self._frame_img)
        self._hist = HistPlotWidget(0)

        self._frame_slider = QtWidgets.QSlider(Qt.Horizontal)
        self._frame_slider.setMinimum(0)
        self._frame_slider.setMaximum(pd.frame_count - 1)
        self._frame_slider.valueChanged.connect(self.set_frame_index)

        layout.addWidget(self._cpu, 0, 0)
        layout.addWidget(self._gpu, 1, 0)
        layout.addWidget(self._frame_widget, 0, 1)
        layout.addWidget(self._hist, 1, 1)
        layout.addWidget(self._frame_slider, 2, 0, 1, 2)

        self.setLayout(layout)

    def set_frame_index(self, frame_index):
        self._cpu.set_frame_index(frame_index)
        self._gpu.set_frame_index(frame_index)
        self._hist.set_frame_index(frame_index)

        im = Image.open("{}frames/{}.bmp".format(profiling_dir, frame_index))
        im = np.array(im)
        im = np.transpose(im, axes = [1, 0, 2])
        im = np.flip(im, axis = 1)

        self._frame_img.setImage(im)

app = QtWidgets.QApplication([])
main = MainWidget()
main.show()
app.exec_()

