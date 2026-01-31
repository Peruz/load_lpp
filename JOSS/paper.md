---
title: 'Load_LPP: a Rust pipeline to Log, Process, and Plot load time series'
tags:
  - Weighing
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
Load\_LPP addresses the lack of similar libraries and software in hydrological and lysimeter studies.
Such applications face several challenges because of their outdoor location, maintenance (remote and underground installation), load resolution requirements, and high temporal variability of both signal (rain and ET) and noise (wind, temperature, etc.).
Beyond hydrology, the above challenges are common in several other fields and industry.

![Example of usage in edge devices [@peruzzo2024]. \label{fig1}](figure.png)

# Statement of need
Weighing systems are critical components in many fields, from drug dosing to large systems in farming, construction, mining, and smart technologies.
The variety of applications results in diverse technical challenges: uneven load distributions, mechanical stresses, temperature effects, and vibrations [@peters2016; @tiboni2020].
The temporal dynamics of such issues give variable levels of signal and noise.
In addition, both temporal resolution and measurement time vary significantly, e.g., long monitoring measuring fast load changes [@jacob2010; @lin2022].
Such challenges have been addressed with mechanical and processing solutions, the latter being preferred because of cost limitations.

# State of the field
The field of weighing systems is dominated by commercial solutions, which have common limitations for research applications:

1. Require specific loggers or laptops (e.g., GUIs).
2. Hence, they limit the control over long-term monitorings (e.g., operative system schedules).
3. Provide limited processing and automation functionalities, while sometimes also limiting the flexibility in recording true-raw data.
4. Significant processing overhead because of manufacturer-specific formats and functionalities, particularly when multiple systems are used across different projects.
5. Are not or hardly extendable.
6. Are not suited for large data sets, from the processing and visualization of large files, to numerical optimization.
7. They are not open source and may conflict with scientific best practices, depending on their complexity level (e.g., point 3).

Research contributions focus on the scientific and theoretical aspects of the weighing designs, often prioritizing scientific aspects for publication.
As such, they do not focus and provide the underlying software solutions.
Other studies focus on the processing theoretical aspects, but rarely provide the actual codes.
To the best of our knowledge, there is no equivalent data pipeline integrating relevant processing functionalities in a system-agnostic solution.

# Software design
The logging functionality (`load_log`) manages a TCP stream of data and settings between a general logging device and a digital amplifier.
Load\_LPP automatically recovers the connection in case of timeouts, power loss, or TCP errors.
This supports more autonomous and remote scientific setups, often running on limited power supplies.
Load\_LPP uses timezone-aware datetime and also handles clock variations over the year (RFC 3339 - ISO 8601).
The command-line help documents the connection and logging configurations, and provides convenient defaults.
Thanks to the limited dependencies and [Rust](https://rust-lang.org/) low-level management of resources, the compiled executable can run on minimal-edge devices.

The processing functionalities include generic and specific time-series operations, organized in `load_process`.
First, it checks the continuity and order of the time series, handling time zones and clock variations.
The second processing step removes periods specified by the user (maintenance, etc.) with a text file.
The filtering also automatically detects and adds anomalous periods to the above file.
Then, a range-based filter removes possible outliers.
For smoothing and gap-filling the data, a weighted moving average is implemented with user-defined width and weight distribution.
A maximum missing weight is defined so that the central value being interpolated is left to NaN when too many measurements, or associated weight, are missing.
Alternatively, an adaptive-window solution addresses the temporal variability of noise and load dynamics (`AWAT`).
This calculates the relative levels of signal and noise with a polynomial regression of the windowed data [@hannes2015; @peters2014].
The optimal window width is calculated based on the Akaike's information criterion [@akaike1974]:
significant and fast load changes favor shorter windows but this is balanced by the relative relevance of the noise, which favors longer windows.
All operations are parallelized using multi-threading and SIMD because of the expected long time series, high temporal resolution, and large smoothing windows.

The visualization executable provides a convenient and interactive visualization of the data, reading both raw and processed data.
The visualization smoothly handles the expected large time series, up to some millions of measurements.

The library includes several tests, run with the Rust standard `cargo test`.

# Research impact statement
The library was used by @peruzzo2024 and @mary2023.
A third related work is in preparation.
Considering the number of similar hydrological studies, the software publication could lead to its adoption from other research groups.
More in general, the number of Rust libraries has been increasing, also supported by public policies [@perkel2020; @schueller2022].
Load\_LPP could serve as a reference because the language ecosystem for data management is relatively young and there are not many feature-complete data pipelines.
In this regard, the Load\_LPP has more than 500 downloads from its official Rust installation webpage. 

# AI usage disclosure
No generative AI tools were used in the development of this software, the writing of this manuscript, or the preparation of supporting materials.

# Acknowledgements
We acknowledge the support of the Advanced Research Projects Agency - Energy, Rhizosphere Observations Optimizing Terrestrial Sequestration.
We also acknowledge the Watershed Function Scientific Focus Area funded by the U.S. DOE, award DE-AC02-05CH11231.
