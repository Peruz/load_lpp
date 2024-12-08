---
title: 'Load_LPP: a Rust pipeline to Log, Process, and Plot load time series'
tags:
  - Instrumentation
  - TCP
  - Data logger
  - Time series
  - Monitoring
  - Parallelization
  - Rust
authors:
  - name: Luca Peruzzo^[first author]
    orcid: 0000-0002-4065-8910
    affiliation: "1", "2"
  - name:  Chunwei Chou
    affiliation: "2"
  - name: Mary Benjamin
    affiliation: "1", "3"
  - name: Tonelato Irene
    affiliation: "4"
  - name: Yuxin Wu
    affiliation: "2"
affiliations:
 - name: Universisty of Padova, Department of Geosciences, Italy.
   index: 1
 - name: Environmental and Earth Sciences Area, Lawrence Berkeley National Laboratory, California, USA.
   index: 2
 - name: Institute of Agricultural Sciences, Madrid, ES
   index: 3
 - name: Independent contributor
   index: 4
date: 12 February 2022
bibliography: paper.bib

# Optional fields if submitting to a AAS journal too, see this blog post:
# https://blog.joss.theoj.org/2018/12/a-new-collaboration-with-aas-publishing
aas-doi: 10.3847/xxxxx <- update this with the DOI from AAS once you know it.
aas-journal: Astrophysical Journal <- The name of the AAS journal.
---

# Key Points and Ideas
TNeed for reliability and/or performance in distributed sensing capable of continuous monitoring.
For example, edge devices and system of load cells, and engineering with quality standards.
The safely-compiled rust binaries provide more flexibility of deployment.
At the same time, it is relatively maintainable (high level convenient tools and compiler checks and cargo)
which makes it good for scientists and other people with reduce programming background.
However, this safe and high-level still provide access low-level optimization, such as allocation and parallelization.

Highlight the general relevance of load cells.

Mainstream commercial solutions, often proprietary, imply the use of GUIs which hinders their you on edge devices and automation.
In particular, commercial solutions offer limited automatic-processing functionalities.
Here we:....

The logging app is detached, so that alternative loggers could be added.

I wrote the pipeline, other co-authors contributed by highliting the needs, validation, editing, and funding.

Add a figure with the processing or the load cells?
Plot loads anomaly and add them as vertical bars.

The three steps are split so that one could only use the processing on a file originated from different applications (e.g., using Python or a proprietery application).

### Anomalies
Events: like a single outlier, a car, etc.; based on the difference between successive loads.
Period: like maintenance or temporal removal of something; based on the IQR range and MAD over a moving window, to avoid trigering for a signle outlier.

## Plot
Add anomalies and bad datetimes to the plot.
Add legend, it makes it more complete.


# Summary
Load_LPP is a Rust pipeline for measuring, processing, and visualize data from load cells.
Weighing system of load cells are everywhere.
Regarding science needs, both laboratory-based and outdoor research rely on load cells for reference values and time series used to contrain reaction and fluxes.
Industry, healthcare, and logistic sectors also strongly rely on the weighing system [@cite:year].
The importance of reliable, automated, and performant weighing system is confirmed by the increasing availability of smart system and research efforts on processing algorithms.
However, these research advances do not find a convenient way of adoption, but remain mostly bound to proprietery hardware-software boundles.
Commonly the need includes the automatic recording and processing of the load.
In this sense, common commercial applications have some limitations:
1. Often require a specific logger or a laptop.
2. Provide only minimum garancies over long term periods.
3. Procide limited processing and automation functionalities.
4. Requeire significant preprocessing and post-processing steps because of the device-specific formats and functionalities.
5. Are not, or hardly, extendible.
6. Are not suited for large data sets (long term stability with minimal instrumentation such as a raspberry pi and automated and optimized processing functionalities).

In general terms, these limitations are also found and address in similar fields.
Accelerometers 
other field (accelerometers, ...).
At the best of our knowledge, we do not know of and equvalent for load cells.
The only exception being hx711 (url, limitation: language and documentation, no processing).

Despite the growing popularity of RUST language, full real-world examples remain relatively rare.
Here we show....

The need for and open, extendible, and ... framework that goes beyond the limitations 



This is For this reason, commercial scales increasily come with 
Virtua




is a very common need in many scientific fields.
In fact, the use of load cells has been pervasive in virtually every scientific 



The forces on stars, galaxies, and dark matter under external gravitational
fields lead to the dynamical evolution of structures in the universe. The orbits
of these bodies are therefore key to understanding the formation, history, and
future state of galaxies. The field of "galactic dynamics," which aims to model
the gravitating components of galaxies to study their structure and evolution,
is now well-established, commonly taught, and frequently used in astronomy.
Aside from toy problems and demonstrations, the majority of problems require
efficient numerical tools, many of which require the same base code (e.g., for
performing numerical orbit integration)

# Statement of need



`Gala` is an Astropy-affiliated Python package for galactic dynamics. Python
enables wrapping low-level languages (e.g., C) for speed without losing
flexibility or ease-of-use in the user-interface. The API for `Gala` was
designed to provide a class-based and user-friendly interface to fast (C or
Cython-optimized) implementations of common operations such as gravitational
potential and force evaluation, orbit integration, dynamical transformations,
and chaos indicators for nonlinear dynamics. `Gala` also relies heavily on and
interfaces well with the implementations of physical units and astronomical
coordinate systems in the `Astropy` package [@astropy] (`astropy.units` and
`astropy.coordinates`).

`Gala` was designed to be used by both astronomical researchers and by
students in courses on gravitational dynamics or astronomy. It has already been
used in a number of scientific publications [@Pearson:2017] and has also been
used in graduate courses on Galactic dynamics to, e.g., provide interactive
visualizations of textbook material [@Binney:2008]. The combination of speed,
design, and support for Astropy functionality in `Gala` will enable exciting
scientific explorations of forthcoming data releases from the *Gaia* mission
[@gaia] by students and experts alike.

# Mathematics

Single dollars ($) are required for inline mathematics e.g. $f(x) = e^{\pi/x}$

Double dollars make self-standing equations:

$$\Theta(x) = \left\{\begin{array}{l}
0\textrm{ if } x < 0\cr
1\textrm{ else}
\end{array}\right.$$

You can also use plain \LaTeX for equations
\begin{equation}\label{eq:fourier}
\hat f(\omega) = \int_{-\infty}^{\infty} f(x) e^{i\omega x} dx
\end{equation}
and refer to \autoref{eq:fourier} from text.

# Citations

Citations to entries in paper.bib should be in
[rMarkdown](http://rmarkdown.rstudio.com/authoring_bibliographies_and_citations.html)
format.

If you want to cite a software repository URL (e.g. something on GitHub without a preferred
citation) then you can do it with the example BibTeX entry below for @fidgit.

For a quick reference, the following citation commands can be used:
- `@author:2001`  ->  "Author et al. (2001)"
- `[@author:2001]` -> "(Author et al., 2001)"
- `[@author1:2001; @author2:2001]` -> "(Author1 et al., 2001; Author2 et al., 2002)"

# Figures

Figures can be. included like this
![Caption for example figure.\label{fig:example}](figure.png)
and referenced from text using \autoref{fig:example}.

Figure sizes can be customized by adding an optional second parameter:
![Caption for example figure.](figure.png){ width=20% }

# Acknowledgements

We acknowledge contributions from Brigitta Sipocz, Syrtis Major, and Semyeong
Oh, and support from Kathryn Johnston during the genesis of this project.

# Availability
Flintec_LPP is available at the github repository or with: cargo install flintec_lpp

# References:
