---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

# Longevity

Health records need to last a long time. Sometimes, they exist before a person is born and last well after death.

However, healthcare software is a different matter.

Like all software, there is a lifecycle to its existence. Some companies fail, some grow, and some get acquired by bigger companies. Software tends to last less time than humans.

In the current closed-source EHR world, users and managers of patient data do not have access to the source code. Due to the inseparable tight coupling of patient data record structure to the database used inside the software platform, it becomes challenging to salvage the data in a usable form once the software dies.

GitEHR seeks to separate the clinical record from the software used to view and manage it. It will ensure patient records stand alone, separate from the database structure of any single proprietary software supplier.

Using time-tested, simple technologies - such as flat files, directories, and disks - helps reduce GitEHR's dependence on the latest **new and shiny** trend. It ensures that once viewing and editing software reaches its inevitable end of life, new software can seamlessly replace the old without affecting clinical care.

## Open formats outlive proprietary ones

The practical threat to a long-lived record is not disk failure but **format obsolescence**. A proprietary binary database from 2005 whose vendor has since disappeared may have to be reverse-engineered to recover the data inside it; plain text from decades earlier is still trivially readable. This is why digital archivists - the Digital Preservation Coalition, the Library of Congress - recommend capturing data in **vendor-neutral, open, text-based formats** (CSV, or self-contained open formats like SQLite) for long-term preservation, and why so many healthcare organisations struggle to move legacy data into new systems: it is trapped in formats nothing new can read.

A database *can* endure, if it is open and actively maintained. The US Veterans Health Administration's VistA system stored veterans' records in a MUMPS-based database for nearly forty years - possible precisely because the system was open-source and well-documented. But even VistA eventually faces an "archiving problem": migrating decades of data to whatever comes next. A file-based archive sidesteps that question, because the format a future system has to read is just text.
