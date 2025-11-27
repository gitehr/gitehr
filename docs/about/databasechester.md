# Databasechester

To try to explain why I have come to the conclusion that #DatabasesAreHarmful and #FilesAreTheAnswer

Join me in imagining my fictional city Databasechester in which there is *no such thing* as a document file format. Every organisation keeps its internal documents in a database inside its organisation. And of course this works adequately well, *inside their organisation*. Putting aside the fact that they are locked in to their vendor

But now, in our fictional city, the local Council want to send documents to the Fire Department. But the Fire Department have their own, different, document management database. So the Council have to commission development of some software to 'translate' a Council document into a Fire Department document.

It kind of works, but it was expensive to have built, and there are features in the Council's system that can't be represented accurately in the Fire Department system, so the translation is 'lossy'. But in some way the bulk of the communication gets through and the Council can now send a document to the Fire Department.

So far so awful. But now the Fire Department needs to send a reply. Problem. They **also** now need to commission a piece of software to convert from their internal document database to the Council's. More expense and difficulty ensue, but after a while the Fire Department finally now has the ability to communicate *back* to the Council.

But - there is also the Police Department to communicate with. More software is commissioned by both organisations.

As the Council needs to communicate with more and more departments and teams, the amount of work increases rapidly, creating huge costs and delays, and also creating lossy, poor quality communication.

Nobody would be surprised if a city which managed simple documents like this could achieve very little, despite huge amounts of money being spent on interoperability.

Is this starting so sound familiar to anyone working in health tech? The absence of any kind of file format for medical records has created this exact scenario within the NHS. The NHS is a city of databases, with no document file format.
