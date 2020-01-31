import numpy as np
from PyQt5 import QtCore, QtGui, QtWidgets
from PyQt5.QtCore import Qt
import pyqtgraph as pg
from profiling_data import ProfilingData
from PIL import Image

pg.setConfigOption('antialias', True)
pg.setConfigOption('background', 'w')
pg.setConfigOption('foreground', 'k')

profiling_dirs = ["../profiling/ortho_encl/", "../profiling/persp_encl/"]

pds = [ProfilingData(profiling_dir) for profiling_dir in profiling_dirs]


class SamplePlotWidget(QtWidgets.QWidget):
    def __init__(self,
                 pd_index=0,
                 sample_index=0,
                 cpugpu_index=0,
                 frame_index=0,
                 *args,
                 **kwargs):
        super(SamplePlotWidget, self).__init__(*args, **kwargs)

        self.pd_index = pd_index
        self.sample_index = sample_index
        self.cpugpu_index = cpugpu_index
        self.frame_index = frame_index

        self._pd_combo = QtWidgets.QComboBox()
        self._pd_combo.addItems([pd.profiling_dir for pd in pds])
        self._pd_combo.setCurrentIndex(pd_index)
        self._pd_combo.currentIndexChanged.connect(self.set_pd_index)

        self._sample_combo = QtWidgets.QComboBox()
        self._sample_combo.addItems(self.pd().sample_names)
        self._sample_combo.addItems(['Shade Ops', 'Light Calcs'])
        self._sample_combo.setCurrentIndex(cpugpu_index)
        self._sample_combo.currentIndexChanged.connect(self.set_sample_index)

        self._cpugpu_combo = QtWidgets.QComboBox()
        self._cpugpu_combo.addItems(['CPU', 'GPU'])
        self._cpugpu_combo.setCurrentIndex(cpugpu_index)
        self._cpugpu_combo.currentIndexChanged.connect(self.set_cpugpu_index)

        self._plot = pg.PlotWidget()

        h_layout = QtWidgets.QHBoxLayout()
        h_layout.addWidget(self._pd_combo)
        h_layout.addWidget(self._sample_combo)
        h_layout.addWidget(self._cpugpu_combo)

        v_layout = QtWidgets.QVBoxLayout()
        v_layout.addLayout(h_layout)
        v_layout.addWidget(self._plot)
        self.setLayout(v_layout)

        self._update_pd()

    def pd(self):
        return pds[self.pd_index]

    def set_pd_index(self, pd_index):
        print("set_pd_index called {}".format(pd_index))
        if self.pd_index == pd_index:
            return

        self.pd_index = pd_index
        self._update_pd()

    def _update_pd(self):
        self._plot.clear()
        self._plot_runs = [
            pg.PlotDataItem() for _ in range(0,
                                             self.pd().run_count)
        ]
        for pdi in self._plot_runs:
            self._plot.addItem(pdi)
        for run_index in range(1, self.pd().run_count):
            self._plot_runs[run_index].setPen(
                color=QtGui.QColor(127, 127, 127, 255))
        self._plot_runs_med = pg.PlotDataItem()
        self._plot.addItem(self._plot_runs_med)
        self._plot_runs_med.setPen(color=QtGui.QColor(100, 100, 255, 255))
        self._plot_frame_line = pg.InfiniteLine(
            self.frame_index, bounds=[0, self.pd().frame_count - 1])
        self._plot.addItem(self._plot_frame_line)

        sample_index = self.sample_index
        self._sample_combo.clear()
        self._sample_combo.addItems(self.pd().sample_names)
        self._sample_combo.addItems(['Shade Ops', 'Light Calcs'])
        self._sample_combo.setCurrentIndex(sample_index)

        cpugpu_index = self.cpugpu_index
        self._cpugpu_combo.clear()
        self._cpugpu_combo.addItems(['CPU', 'GPU'])
        self._cpugpu_combo.setCurrentIndex(cpugpu_index)

        self.update_plot()

    def set_sample_index(self, sample_index):
        self.sample_index = sample_index
        self.update_plot()

    def set_cpugpu_index(self, cpugpu_index):
        self.cpugpu_index = cpugpu_index
        self.update_plot()

    def set_frame_index(self, frame_index):
        self.frame_index = frame_index
        self._plot_frame_line.setPos(self.frame_index)

    def update_plot(self):
        for pdi in self._plot_runs:
            pdi.clear()
        self._plot_runs_med.clear()

        if self.sample_index < self.pd().sample_count:
            for run_index in range(1, self.pd().run_count):
                self._plot_runs[run_index].setData(y=self.pd(
                ).deltas[run_index, :, self.sample_index, self.cpugpu_index])
            y = np.min(
                self.pd().deltas[1:, :, self.sample_index, self.cpugpu_index],
                axis=0,
                keepdims=True)
            self._plot_runs_med.setData(y=y[0, :])
        elif self.sample_index < self.pd().sample_count + 2:
            basic_sample_index = self.sample_index - self.pd().sample_count
            for basic_buffer_index in range(0, self.pd().basic_buffer_count):
                self._plot_runs[basic_buffer_index].setData(y=self.pd(
                ).basic_buffers[:, basic_buffer_index, basic_sample_index])


class HistPlotWidget(QtWidgets.QWidget):
    def __init__(self,
                 pd_index=0,
                 hist_index=0,
                 frame_index=0,
                 *args,
                 **kwargs):
        super(HistPlotWidget, self).__init__(*args, **kwargs)

        self.pd_index = pd_index
        self.hist_index = hist_index
        self.frame_index = frame_index

        self._pd_combo = QtWidgets.QComboBox()
        self._pd_combo.addItems([pd.profiling_dir for pd in pds])
        self._pd_combo.setCurrentIndex(pd_index)
        self._pd_combo.currentIndexChanged.connect(self.set_pd_index)

        self._hist_combo = QtWidgets.QComboBox()
        self._hist_combo.addItems([
            'Fragments per cluster', 'Light indices per cluster',
            'Light indices per fragment'
        ])
        self._hist_combo.currentIndexChanged.connect(self.set_hist_index)

        self._plot_hist = pg.PlotDataItem()

        self._plot = pg.PlotWidget()
        left_axis = self._plot.getAxis('left')
        left_axis.enableAutoSIPrefix(enable=False)
        self._plot.addItem(self._plot_hist)

        g_layout = QtWidgets.QGridLayout()
        self._cluster_count_label = QtWidgets.QLabel()
        self._light_indices_count_label = QtWidgets.QLabel()
        self._fragment_count_label = QtWidgets.QLabel()
        g_layout.addWidget(QtWidgets.QLabel("visible cluster count"), 0, 0)
        g_layout.addWidget(self._cluster_count_label, 0, 1)
        g_layout.addWidget(QtWidgets.QLabel("total light indices"), 1, 0)
        g_layout.addWidget(self._light_indices_count_label, 1, 1)
        g_layout.addWidget(
            QtWidgets.QLabel("total fragments in cluster space"), 2, 0)
        g_layout.addWidget(self._fragment_count_label, 2, 1)

        h_layout = QtWidgets.QHBoxLayout()
        h_layout.addWidget(self._pd_combo)
        h_layout.addWidget(self._hist_combo)

        v_layout = QtWidgets.QVBoxLayout()
        v_layout.addLayout(h_layout)
        v_layout.addLayout(g_layout)
        v_layout.addWidget(self._plot)
        self.setLayout(v_layout)

        self.update_plot()

    def set_pd_index(self, pd_index):
        if self.pd_index == pd_index:
            return
        self.pd_index = pd_index
        self.update_plot()

    def set_hist_index(self, hist_index):
        if self.hist_index == hist_index:
            return
        self.hist_index = hist_index
        self.update_plot()

    def update_plot(self):
        pd = pds[self.pd_index]

        if self.hist_index == 0:
            data = pd.cluster_buffers[:, 0, 256:512]
            all_min = np.min([np.min(pd.cluster_buffers[:, 0, 256:512]) for pd in pds])
            all_max = np.max([np.max(pd.cluster_buffers[:, 0, 256:512]) for pd in pds])
            x = np.arange(1, 33, dtype=np.uint64)
            y = data[self.frame_index, :]
            self._plot.setYRange(all_min, all_max)
            self._plot.setLabel('left', 'Cluster Count')
            self._plot.setLabel('bottom', 'log_2(Fragments)')

        elif self.hist_index == 1:
            data = pd.cluster_buffers[:, 0, 512:768]
            all_min = np.min([np.min(pd.cluster_buffers[:, 0, 512:768]) for pd in pds])
            all_max = np.max([np.max(pd.cluster_buffers[:, 0, 512:768]) for pd in pds])
            x = 8 * np.arange(-0.5, 32.5)
            y = data[self.frame_index, :]
            self._plot.setYRange(all_min, all_max)
            self._plot.setLabel('left', 'Cluster Count')
            self._plot.setLabel('bottom', 'Light Count')

        elif self.hist_index == 2:
            data = pd.cluster_buffers[:, 0, 768:1024]
            all_min = np.min([np.min(pd.cluster_buffers[:, 0, 768:1024]) for pd in pds])
            all_max = np.max([np.max(pd.cluster_buffers[:, 0, 768:1024]) for pd in pds])
            x = 8 * np.arange(-0.5, 32.5)
            y = data[self.frame_index, :]
            self._plot.setYRange(all_min, all_max)
            self._plot.setLabel('left', 'Fragment Count')
            self._plot.setLabel('bottom', 'Light Count')

        self._plot_hist.setData(x, y, stepMode=True)

        self._cluster_count_label.setText("{}".format(
            pd.cluster_buffers[self.frame_index, 0, 0]))
        self._light_indices_count_label.setText("{}".format(
            pd.cluster_buffers[self.frame_index, 0, 1]))
        self._fragment_count_label.setText("{}".format(
            np.sum(pd.cluster_buffers[self.frame_index, 0, 768:1024])))

    def set_frame_index(self, frame_index):
        self.frame_index = frame_index
        self.update_plot()


class MainWidget(QtWidgets.QWidget):
    def __init__(self, *args, **kwargs):
        super(MainWidget, self).__init__(*args, **kwargs)

        layout = QtWidgets.QGridLayout()

        self._cpu = SamplePlotWidget(cpugpu_index=0)
        self._gpu = SamplePlotWidget(cpugpu_index=1)
        self._frame_img = pg.ImageItem()
        self._frame_widget = pg.GraphicsLayoutWidget()
        self._frame_vb = self._frame_widget.addViewBox(row=1, col=1)
        self._frame_vb.setAspectLocked()
        self._frame_vb.addItem(self._frame_img)
        self._hist = HistPlotWidget(0)

        self._frame_slider = QtWidgets.QSlider(Qt.Horizontal)
        self._frame_slider.setMinimum(0)
        self._frame_slider.setMaximum(pds[0].frame_count - 1)
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

        im = Image.open("{}frames/{}.bmp".format(pds[0].profiling_dir,
                                                 frame_index))
        im = np.array(im)
        im = np.transpose(im, axes=[1, 0, 2])
        im = np.flip(im, axis=1)

        self._frame_img.setImage(im)


app = QtWidgets.QApplication([])
main = MainWidget()
main.show()
app.exec_()
