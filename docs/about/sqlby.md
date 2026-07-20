# SQLby

To try to illustrate how 'Databases Are Harmful' in healthcare, I've been using the example of file-based interoperability elsewhere in tech.

Let's imagine a small fictional Yorkshire town of SQLby, in which there is *no such thing* as a document file format. Every organisation in SQLby keeps its internal documents in a proprietary database format, in their 'Document Management System' - or DMS - inside its organisation. They're so proud of their 'go-live' that they even put out a press release when they did it. It's an off-the-shelf DMS, but of course they've heavily customised it locally because they are a unique and special organisation. And of course, for a while, this works adequately enough, *inside their organisation*.

But SQLby Town Council now need to send a letter to the SQLby Fire Department. The Fire Department have their own, different, Document Management System. The only way to send a letter to the Fire Department is for the Council to pay for some software to 'translate' a Council DMS letter into a Fire Department DMS letter. The Council procure a 'solution' and it's installed after 9 months development delay.

It *kind of* works... but it was expensive to have built, and there are some DMS features in the Council's system that can't be represented in the Fire Department Document Management System, so the translation is 'lossy' and imperfect. But in some way, the bulk of the communication gets through and the Council can now send a letter to the Fire Department.

So far so awful. But now the Fire Department needs to send a reply to the Council. That's a problem because the Fire Dept. **also** now need to commission a piece of software to convert from their internal letter format to the Council's letter format. More cost and difficulty ensue, and after a while the Fire Department finally now has the ability to communicate *back* to the Council, after a fashion.

But then both organisations realise that there is also the Police Department to communicate with. More complex integration work is commissioned by both the Council and the Fire Dept. They can't even share the same adapters because all the DMSs are proprietary and any organisation sharing details of the internal workings of their DMS might get sued by their supplier. But they can each quite safely pay a fortune for the separate integration work to be done so that they can each communicate with the Police Dept.

As the Council needs to communicate with more and more departments and teams, the amount of work increases exponentially (for those who want to know - the number of connectors required equals `2^(n-1)`, where `n` is the number of DMSs involved), creating huge costs and delays, and also creating lossy, poor quality communication. They spend more money on integration of data than they do on actually **doing things**.

---

Nobody would be surprised if a town which had decided to manage letters this way would achieve very little of practical use, despite huge amounts of money being spent on 'interoperability'. **Is this starting to sound familiar to anyone working in healthcare?** The absence of any kind of file format for medical records has created this exact scenario all across healthcare. And we're looking in the wrong place for the solution - point-to-point EPR interoperability is expensive, lossy, and effectively impossible to achieve at scale. Bigger silos are just deferring solving the problem for another generation.

### 'Make A Bigger Silo'

Orthodox solutions to this problem such as Regional Care Records and Single Patient Records do not really solve it, and in fact they make it a bigger problem. Having a bigger silo just moves the boundary from Hospital:Hospital up to Country:Country - it kicks the can down the road. That interoperability problem never went away, you just made a bigger silo.

Additionally, most Regional Care Record and Single Patient Record programmes only *look* like 'one record'. Under the hood, for commercial, political, technical, and Information Governance reasons, they are usually forced to rely on point-to-point data connectors and translators, so what appears to be a 'joined-up single patient view' is actually a lot of glitter and chrome over a backend made of duct tape, string, and quantities of Java code hundreds of times above the lethal dose.


### The Real World
In the real world (outside healthcare tech) we take this kind of 'organisation-to-organisation letter interoperability' for granted, because we're so used to it existing and working well. It hasn't always existed and those open file formats were hard-fought-for, but now we have them, nobody is suggesting to go back.

Healthcare has become a country filled with SQLby towns, unable to transfer critical data because it has **no medical records file format**. Worse, it doesn't even want to know that it needs one.

### What GitEHR IS NOT Saying

For absolute clarity at this point I will make it clear that the above SQLby vignette is a light-hearted thought experiment to help explain conceptually why we have had so much difficulty with healhtcare interoperability, and yet there are **other** areas of tech in which we've completely solved those same problems. 

* We are not saying that GitEHR's solution is a single logical file.
* We are not saying that emailing files around will solve healthcare interoperability.
* We are not saying it is advisable to have uncontrolled copied of documents floating around an organisation

GitEHR is a much more elegant solution, based on the tools that software engineers use every day across the world to collaborate on code. The 'healthcare information management' problem has already been solved, we just needed to look outside of healthcare for that solution. The next pages in this documentation site will explain what that solution is, how it works, and how we created it.
