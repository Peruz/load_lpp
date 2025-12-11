---
title: 'Load_LPP: a Rust pipeline to Log, Process, and Plot load time series'
tags:
  - Weighing system
  - Load cell
  - Data logger
  - Time series
  - Monitoring
  - Parallelization
  - Rust
  - TCP
authors:
  - name: Luca Peruzzo^[first author]
    orcid: 0000-0002-4065-8910
    affiliation: "1, 2"
  - name:  Chunwei Chou
    orcid: 0000-0002-9600-4270
    affiliation: "2"
  - name: Irene Tonelato
    affiliation: "3"
  - name:  Yuxin Wu
    orcid: 0000-0002-6953-0179
    affiliation: "2"
affiliations:
 - name: University of Padova, Department of Geosciences, Italy.
   index: 1
 - name: Environmental and Earth Sciences Area, Lawrence Berkeley National Laboratory, California, USA.
   index: 2
 - name: Individual contributor
   index: 3
date: 29 July 2025
bibliography: paper.bib
---


# Summary
Load\_LPP is a data pipeline for Logging, Processing, and Plotting time series from weighing systems.
It addresses the lack of similar libraries and software in hydrological studies targeting rain and evapotranspiration through the monitoring of the water-load changes.
Such applications face several challenges because of their outdoor location, challenging maintenance (remote and underground installation), load resolution requirements, and high temporal variability of both signal (rain and ET) and noise (wind, temperature, etc.).
Beyond hydrology, the above challenges are common issues in several other fields where precise weighing monitoring is needed.

# Mention
The project was used by @peruzzo2024 and @mary2023 (Figure 1).
A third related work is in preparation by the same research group.
Considering the number of similar hydrological studies, the software publication could lead to its adoption from other research groups.
More in general, the number of Rust libraries has been increasing, also supported by some public policies [@matsakis2014; @perkel2020; @schueller2022].
Load\_LPP could also serve as a reference because the language ecosystem for data management is relatively young and there are not many complete data pipelines.

![Example of usage in edge device, as from previous scientific studies [@peruzzo2024]. \label{fig1}](figure.png)

# Statement of need
Weighing systems of load cells are critical and safety components in many fields, from drug dosing to large systems in farming, construction, and mining.
Recent use of weighing systems relates to smart technologies, in smart cities, healthcare, transportation and logistics.
The variety of applications results in diverse technical challenges: uneven load distributions, mechanical stresses, temperature effects, and vibrations [@peters2016; @tiboni2020].
The temporal dynamics of such issues result in variable levels of both signal (i.e., load changes) and noise.
In addition, both temporal resolution and measurement time can vary significantly, e.g., long monitoring measuring fast load changes [@jacob2010; @lin2022].
Such challenges have been addressed with a combination of mechanical and processing solutions, the latter often being preferred because of cost limitations.

In this sense, commercial systems have common limitations:

1. Require specific loggers or laptops (e.g., GUIs).
2. The reliance on specific loggers and laptops also limits the control over long-term monitorings (e.g., operative system schedules).
3. Provide limited processing and automation functionalities, while sometimes also limiting the flexibility in recording true-raw data.
4. Significant processing overhead because of manufacturer-specific formats and functionalities, particularly when multiple systems are used across different projects.
5. Are not, or hardly, extendable.
6. Are not suited for large data sets, from the processing and visualization of large files, to the numerical optimization.
7. From a general perspective, they are not open source and thus can conflict with some scientific best practices, depending on their complexity level (e.g., point 3).

# Implementation
Load\_LPP aims to address the above limitations and challenges.
The logging functionality (`load_log`) manages a TCP stream of data and settings between a general logging device and the digital amplifier.
Load\_LPP automatically recovers the connection after possible timeouts, power loss, or TCP errors.
This is in-line with more autonomous and remote scientific setups, often running on limited power supplies.
The Rust [Chrono](https://crates.io/crates/chrono) library is used to time connections and logging.
Data are stored with timezone-aware datetime and also handles clock variations over the year (RFC 3339 - ISO 8601).
The command-line help documents the connection and logging configurations, and provides convenient default values.
The compiled executable requires very limited CPU and memory resources, thanks to the limited dependencies and specific low-level management of memory and connection.
This allows it to run on very minimal and low-cost edge devices.

The processing functionalities cover the common filtering, gap filling, and smoothing steps, as well as more advanced and specific time-series operations.
As for the logging part, the processing functionalities are then organized into an executable (`load_process`).
The first processing step checks the continuity and order of the time series, handling time zones and clock variations.
The second processing step is the removal of periods specified by the user (maintenance, etc.), these are conveniently stored in a plain text file.
The filtering phase also provides an automatic detection of anomalous periods, which can be added to the previous file.
Then, a range-based filter checks whether the individual values are within an accepted range.
At this point, flexible smoothing solutions are available to smooth the data and replace the NaN values by interpolation (gap filling).
A weighted moving average is defined with user-defined width and weight distribution (linear distribution between a central-maximum value and side-minimum weights).
A maximum missing weight is also defined so that the central value being interpolated is left to NaN when too many measurements are missing within the window, accounting by their relative weight.
These operations are parallelized, using both multi-threading and SIMD because of the expected long time series, high temporal resolution, and large smoothing windows.
Finally, an adaptive-window solution is also implemented to address the temporal variability of noise and load dynamics (`AWAT`).
This calculates the relative level of signal and noise based on a polynomial regression of the windowed data [@hannes2015; @peters2014].
The optimal window width is then calculated based on the Akaike's information criterion [@akaike1974]:
strong signal (significant and fast load changes) favors shorter windows to avoid excessive smoothing; however, this is balanced by the relative relevance of the noise, which favors longer windows.

The visualization executable, which allows a quick visualization of the processed data, reading either the raw or processed data.
The Rust `plotly` bindings are used to handle the smooth and interactive visualization of the expected large time series, up to some millions of measurements.

The library includes several tests, run with the Rust standard `cargo test`.

# Acknowledgements
We acknowledge the support of the Advanced Research Projects Agency - Energy, Rhizosphere Observations Optimizing Terrestrial Sequestration (ARPA-E ROOTS).
We also acknowledge the Watershed Function Scientific Focus Area funded by the U.S. Department of Energy, Office of Science, Office of Biological and Environmental Research under award number DE-AC02-05CH11231.
