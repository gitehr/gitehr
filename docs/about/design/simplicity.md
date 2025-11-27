---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

# Simplicity

**The content of the health record should be in plain text, in files on disk**.

**In plain text**: content is readable even in low-tech environments. Ideally, a viewer will render this plain text into a more usable and feature-rich application view. But the text-on-disk view exists as a fallback.

**In files**: files can easily work interoperably across all operating systems.

**On disk**: as opposed to being in a database*.

!!! "Of course, databases are also on disk."
    Though databases also exist on disk, there is a difference between being buried in a database versus having easy access to a file.

## New entries are saved in a new file

Medical practice can usually be distilled to an 'atomic unit of care'. For a clinic visit, a single entry or a few linked entries could summarise the visit.

## GitEHR entries are linkable

As modern clinical records become more complex, clinicians need help finding specific data items.

The Internet has solved this problem by using the URL or Internet 'link'. Somehow, the most basic of technologies has not yet found its way into the current paradigm.
