import matplotlib
import matplotlib.pyplot as plt
import pandas as pd
import sys


font = {
    'family': 'monospace',
    'weight': 'medium',
    'size': 14
}
matplotlib.rc('font', **font)

fi = sys.argv[1]
fo_svg = fi.replace('.csv', '_downsampled10m.svg')
fo_csv = fi.replace('.csv', '_downsampled10m.csv')
data = pd.read_csv(fi, index_col='datetime', parse_dates=True, infer_datetime_format=True)
data10 = data.resample('360min').mean()
data.to_csv(fo_csv)
plt.figure(figsize=(14, 8))
plt.plot(
    data10.index,
    data10.load,
    'o-',
    color='grey',
    linewidth=5,
    markersize=8,
    markerfacecolor='k',
    markeredgecolor='k',
)
plt.ylabel('load [kg]')
plt.grid()
plt.tight_layout()
plt.savefig(fo_svg)
plt.show()
