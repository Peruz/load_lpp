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
  - name:  Yuxin Wu
    orcid: 0000-0002-6953-0179
    affiliation: "2"
  - name: Tonelato Irene
    affiliation: "3"
affiliations:
 - name: University of Padova, Department of Geosciences, Italy.
   index: 1
 - name: Environmental and Earth Sciences Area, Lawrence Berkeley National Laboratory, California, USA.
   index: 2
 - name: Independent contributor
   index: 3
date: 24 July 2025
bibliography: paper.bib
---


# Summary
Load\_LPP is a data pipeline for Logging, Processing, and Plotting time series from weighing systems.
Load\_LPP was developed to offer a complete but also flexible and extendable pipeline (see implementation section).
It was designed to address the lack of similar libraries and ready-to-use executables in scientific hydrological studies targeting rain and evapotranspiration fluxes through the monitoring of the water-load changes, e.g., using lysimeters and rhizotrons.
Such applications face several challenges because of their outdoor location, expected low and difficult maintenance (remote and underground installation), load resolution requirements, and high temporal variability of both signal (water changes associated with rain and ET) and noise (wind and temperature changes in primis).
Beyond hydrology, the above challenges are common issues in several other fields where precise weighing monitoring is needed, as described in the following section.
Load\_LPP is written in Rust and takes advantage of the low-level optimization and general reliability of the compiled software, which become relevant in the expected applications.

# Mention
The project was used during the lysimeter and rhizotron studies by @peruzzo2024 and @mary2023 (Figure 1).
A third related work is in preparation by the same research group.
Considering the number of similar hydrological studies, the software publication could lead to its adoption from other research groups.
More in general, the number of published Rust libraries has been increasing, also supported by some public policies and recommendations [@matsakis2014; @perkel2020; @schueller2022].
Nonetheless, the language ecosystem for data management is relatively young and there are not many complete data pipelines, from logging to visualization.
In this sense, Load\_LPP could also serve as a reference for other software focusing data science and pipelines.

![Example of usage in edge device, as from previous scientific studies [@peruzzo2024]. \label{fig1}](figure.png)

# Statement of need
Weighing systems of load cells have been relevant in many fields, often serving as critical and safety components.
For example, weighing systems have been used to track precise dosages in pharmaceutical manufacturing, feed storage and mixing in agriculture and farming, and larger systems are often used in construction and mining.
Recent use of weighing systems has been linked to the diffusion of smart technologies, for example in smart cities, healthcare, transportation and logistics.
While the variety of applications results in diverse technical challenges, these are typically associated with uneven load distributions, mechanical stresses, temperature effects, and vibrations [@peter2016; @tiboni2020].
In addition, the temporal dynamics of such predisposing factors result in variable levels of both signal (i.e., load changes) and noise (e.g., vibrations), which further complicates the design of optimal weighing systems and processing procedures.
Both temporal resolution and measurement time can vary significantly, with applications that often require the system to reliably function over relatively long periods, from quick weigh-in-motion measurements [@jacob2010; @lin2022] to monitoring of years .
Such issues have been addressed with a combination of mechanical and processing - filtering solutions, the later often being preferred because of cost limitations.

In this sense, commercial systems have some possible limitations:

1. Often require specific loggers or laptops (e.g., GUIs).
2. In turn, the reliance on specific loggers and laptops also limits the control over long-term monitorings (e.g., operative system schedules).
3. Provide limited processing and automation functionalities, while sometimes also limiting the flexibility in recording true-raw data at the desired temporal resolution.
4. Can have significant overhead processing steps because of device- or manufacturer-specific formats and functionalities, particularly when multiple systems are used across different projects.
5. Are not, or hardly, extendable.
6. Are not suited for large data sets, from the simple handling and visualization of large files to the numerical optimization.
7. From a general perspective, the are not open source and thus can conflict with some scientific best practices, depending on their complexity level (e.g., point 3).

# Implementation
Load\_LPP logging functionality, combined with the rich and optimized processing tools, aims to address the above limitations and introduced challenges.
The logging functionality (`load_log`) manages a TCP (Transmission Control Protocol) stream between the logging device and the digital amplifier of the weighing system.
This connection is used to received the data and control the logging settings.
The logging functionality only depends on the network primitives provided by the Rust standard library [std::net](https://doc.rust-lang.org/std/net/index.html) and the [Chrono](https://crates.io/crates/chrono) library.
Among the implemented functionalities, Load\_LPP automatically recovers the connection after possible timeouts and other issues that are not covered by the primitives, and yet frequent in actual applications.
This is in-line with more autonomous and remote scientific setups, often also running on limited power supplies.
The Chrono library to manage the time aspects of the connection and logging schedule.
Note that data are stored and processed with timezone-aware datetime, which also correctly handles clock variations over the year (standard format RFC 3339 - ISO 8601).
The executable offers several configurations, including connection parameters to adapt to different weighing systems and logging parameters, such as measurement frequency and delays.
The configurations are accessible and documented within the command-line help functionality, which also provides convenient default values.
The compiled executable requires very limited CPU and memory resources, thanks to the limited dependencies and careful and low-level management of memory and connection.
This allows it to run on very minimal and low-cost edge devices.
The use of the Rust language offered a good balance of optimization and reliability for such edge applications.

The processing functionalities cover the common filtering, gap filling, and smoothing steps, as well as more specific time-series operations.
As for the logging part, the processing functionalities are then organized into an executable, which eases and organizes the processing steps (`load_process`).
The first processing step checks the continuity and order of the time series, including the automatic handling of time zones and clock variations.
The continuity is automatically checked and, if needed, corrected relative to the minimum temporal interval observed within the time series.
The missing values are filled with NaN.
The second processing step is the removal of periods specified by the user (maintenance, etc.), these are conveniently stored and read from a plain text file.
In addition, the filtering phase also provides an automatic detection of anomalous periods, which can be saved into a file for successive user verification merging the two files.
The straight-forward range-based filter is also implemented, which checks whether the individual values are within an accepted range.
The discarded values are replaced with NaN.
At this point, flexible smoothing solutions are available to smooth the data and replace the NaN values by interpolation (gap filling), when suitable.
A weighted mowing average is defined with user-defined width and weight distribution (linear distribution between a central-maximum value and side-minimum weights).
A maximum missing weight is also defined so that when too many measurements are missing within the window - accounting by their relative weight - the central value being interpolated is left to NaN.
This would warn the user about large data gaps, which are not addressed by simple gap-filling.
Because of the expected long time series and possibly large windows (containing many data), these operations are parallelized, using both multi-threading and single-instruction multiple-data optimizations.
Finally, an adaptive-window solution is also implemented to address the temporal variability of noise and load dynamics (`AWAT`).
This calculates the relative level of signal and noise based on a polynomial regression of the windowed data [@hannes2015; @peter2014].
The optimal window width is then calculated based on the Akaike's information criterion [@akaike1974]:
strong signal (i.e., significant and fast load changes) favors shorter windows to avoid excessive smoothing; however, this is balanced by the relative relevance of the noise, which favors longer windows.

A third and last executable is the plotting part, which allows a quick visualization of the processed data, reading either the raw or processed data.
The Rust `plotly` bindings are used to handle the smooth and interactive visualization of the expected large time series, up to some million of measurements.


# Acknowledgements
We acknowledge the support of the Advanced Research Projects Agency - Energy, Rhizosphere Observations Optimizing Terrestrial Sequestration (ARPA-E ROOTS).
We also acknowledge the Watershed Function Scientific Focus Area funded by the U.S. Department of Energy, Office of Science, Office of Biological and Environmental Research under award number DE-AC02-05CH11231.
