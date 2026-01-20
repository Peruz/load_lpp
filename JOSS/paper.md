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
  - name: Giorgio Cassiani
    orcid: 0000-0002-9060-5606
    affiliation: "1"
  - name:  Yuxin Wu
    orcid: 0000-0002-6953-0179
    affiliation: "2"
affiliations:
 - name: University of Padova, Department of Geosciences, Italy.
   index: 1
 - name: Environmental and Earth Sciences Area, Lawrence Berkeley National Laboratory, USA.
   index: 2
 - name: Individual contributor
   index: 3
date: 29 July 2025
bibliography: paper.bib
---


# Summary
Load\_LPP is a data pipeline for Logging, Processing, and Plotting time series from weighing systems.
Load\_LPP addresses the lack of similar libraries and software in hydrological studies targeting rain and evapotranspiration through the monitoring of the water-load changes.
Such applications face several challenges because of their outdoor location, challenging maintenance (remote and underground installation), load resolution requirements, and high temporal variability of both signal (rain and ET) and noise (wind, temperature, etc.).
Beyond hydrology, the above challenges are common in several other fields and industry.


![Example of usage in edge devices [@peruzzo2024]. \label{fig1}](figure.png)

# Statement of need
Weighing systems are critical components in many fields, from drug dosing to large systems in farming, construction, and mining.
Recent use of weighing systems relates to smart technologies, in smart cities, healthcare, and logistics.
The variety of applications results in diverse technical challenges: uneven load distributions, mechanical stresses, temperature effects, and vibrations [@peters2016; @tiboni2020].
The temporal dynamics of such issues give variable levels of signal (load changes) and noise.
In addition, both temporal resolution and measurement time can vary significantly, e.g., long monitoring measuring fast load changes [@jacob2010; @lin2022].
Such challenges have been addressed with a combination of mechanical and processing solutions, the latter often being preferred because of cost limitations.

Herein, commercial systems have common limitations:

1. Require specific loggers or laptops (e.g., GUIs).
2. The reliance on specific loggers and laptops also limits the control over long-term monitorings (e.g., operative system schedules).
3. Provide limited processing and automation functionalities, while sometimes also limiting the flexibility in recording true-raw data.
4. Significant processing overhead because of manufacturer-specific formats and functionalities, particularly when multiple systems are used across different projects.
5. Are not, or hardly, extendable.
6. Are not suited for large data sets, from the processing and visualization of large files, to numerical optimization.
7. They are not open source and may conflict with scientific best practices, depending on their complexity level (e.g., point 3).

# Software design
Load\_LPP aims to address the above limitations and challenges.
The logging functionality (`load_log`) manages a TCP stream of data and settings between a general logging device and a digital amplifier.
Load\_LPP automatically recovers the connection in case of timeouts, power loss, or TCP errors.
This supports more autonomous and remote scientific setups, often running on limited power supplies.
The Rust [Chrono](https://crates.io/crates/chrono) library is used to time connections and logging.
Load\_LPP uses timezone-aware datetime and also handles clock variations over the year (RFC 3339 - ISO 8601).
The command-line help documents the connection and logging configurations, and provides convenient default values.
Thanks to the limited dependencies and [Rust](https://rust-lang.org/) low-level management of resources, the compiled executable can run on minimal-edge devices.

The processing functionalities include common processing steps and more specific time-series operations.
As for the logging part, the processing functionalities are organized into an executable (`load_process`).
First, it checks the continuity and order of the time series, handling time zones and clock variations.
The second processing step removes periods specified by the user (maintenance, etc.) and stored in a text file.
The filtering also automatically detects anomalous periods, which can be added to the previous file.
Then, a range-based filter checks whether the individual values are within an accepted range.
Flexible solutions are provided to smooth and gap-fill the data.
A weighted moving average is implemented with user-defined width and weight distribution.
A maximum missing weight is defined so that the central value being interpolated is left to NaN when too many measurements, or associated weight, are missing.
These operations are parallelized using both multi-threading and SIMD because of the expected long time series, high temporal resolution, and large smoothing windows.
Finally, an adaptive-window solution is implemented to address the temporal variability of noise and load dynamics (`AWAT`).
This calculates the relative levels of signal and noise with a polynomial regression of the windowed data [@hannes2015; @peters2014].
The optimal window width is calculated based on the Akaike's information criterion [@akaike1974]:
significant and fast load changes favor shorter windows to avoid excessive smoothing; however, this is balanced by the relative relevance of the noise, which favors longer windows.

The visualization executable provides a convenient and interactive visualization of the data, reading both raw and processed data.
The visualization smoothly handles the expected large time series, up to some millions of measurements.

The library includes several tests, run with the Rust standard `cargo test`.

# Research impact statement
The library was used by @peruzzo2024 and @mary2023 (Figure 1).
A third related work is in preparation.
Considering the number of similar hydrological studies, the software publication could lead to its adoption from other research groups.
More in general, the number of Rust libraries has been increasing, also supported by public policies [@matsakis2014; @perkel2020; @schueller2022].
Load\_LPP could serve as a reference because the language ecosystem for data management is relatively young and there are not many feature-complete data pipelines.

# AI usage disclosure
No generative AI tools were used in the development of this software, the writing of this manuscript, or the preparation of supporting materials.

# Acknowledgements
We acknowledge the support of the Advanced Research Projects Agency - Energy, Rhizosphere Observations Optimizing Terrestrial Sequestration.
We also acknowledge the Watershed Function Scientific Focus Area funded by the U.S. Department of Energy, Office of Biological and Environmental Research, award DE-AC02-05CH11231.
