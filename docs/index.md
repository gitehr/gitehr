<div class="gitehr-landing">

  <!-- HERO SECTION -->
  <div class="gitehr-band gitehr-band--hero">
    <div class="gitehr-container">
      <div class="gitehr-hero">
        <div class="gitehr-hero__content">
          <div class="gitehr-kicker">Open Source Standard</div>
          <h1 class="gitehr-hero__title">Decentralised, lossless <span>health records</span></h1>
          <p class="gitehr-hero__subtitle">
            GitEHR is an open source platform powered by Git, cryptography, and open standards. It keeps every clinical change auditable, portable, and owned by the patient.
          </p>
          <div class="gitehr-hero__actions">
            <a href="getting-started.md" class="gitehr-button gitehr-button--primary">Get Started</a>
            <a href="https://github.com/gitehr/gitehr" class="gitehr-button gitehr-button--secondary">View on GitHub</a>
          </div>
        </div>
        <div class="gitehr-hero__image">
          <img src="assets/images/hero-illustration.svg" alt="GitEHR architecture illustration" />
        </div>
      </div>
    </div>
  </div>

  <!-- GETTING STARTED CALLOUT -->
  <div class="gitehr-band gitehr-band--cta">
    <div class="gitehr-container">
      <div class="gitehr-cta">
        <div class="gitehr-cta__content">
          <div class="gitehr-kicker">Developer preview</div>
          <h3>Start with the GUI and repository basics</h3>
          <p>Use the Getting Started guide to run the GUI, open a repository, and learn the on-disk layout.</p>
        </div>
        <div class="gitehr-cta__actions">
          <a href="getting-started.md" class="gitehr-button gitehr-button--primary">Read the guide</a>
          <a href="guides/gui-walkthrough.md" class="gitehr-button gitehr-button--secondary">GUI walkthrough</a>
        </div>
      </div>
    </div>
  </div>

  <!-- VALUE PROP SECTION -->
  <div class="gitehr-band">
    <div class="gitehr-container">
      <h2>The Health Record, Reimagined</h2>
      <p class="gitehr-lead">
        We replaced the siloed database with a distributed ledger. The result is a system that is naturally secure, offline-first, and infinitely portable.
      </p>
      <div class="gitehr-grid gitehr-grid--3">
        <div class="gitehr-card">
          <h3>Patient Owned</h3>
          <p>Records live with the patient, not the institution. Share them across providers, borders, and decades without rebuilding the data.</p>
        </div>
        <div class="gitehr-card">
          <h3>Cryptographic Trust</h3>
          <p>Every entry links to the last one, forming an append-only chain that is tamper-evident and mathematically verifiable.</p>
        </div>
        <div class="gitehr-card">
          <h3>Offline First</h3>
          <p>Git-powered repositories work without constant connectivity. Sync seamlessly whenever a secure link is available.</p>
        </div>
      </div>
    </div>
  </div>

  <!-- AUDIENCE SECTION -->
  <div class="gitehr-band gitehr-band--alt">
    <div class="gitehr-container">
      <h2>Built for the entire ecosystem</h2>
      <div class="gitehr-grid gitehr-grid--3">
        
        <!-- CLINICIANS -->
        <div class="gitehr-audience">
          <div class="gitehr-audience__header">
            <div class="gitehr-audience__icon">C</div>
            <h3>For Clinicians</h3>
          </div>
          <p><strong>See the whole picture.</strong> Access a patient's complete longitudinal history, not just what's in your local system.</p>
          <p>GitEHR provides a unified view of care across specialists and facilities, reducing medical errors and redundant testing.</p>
        </div>

        <!-- PATIENTS -->
        <div class="gitehr-audience">
          <div class="gitehr-audience__header">
            <div class="gitehr-audience__icon">P</div>
            <h3>For Patients</h3>
          </div>
          <p><strong>Your health, in your hands.</strong> You hold the master copy of your record. No more requesting faxed transfers.</p>
          <p>Grant temporary access to new doctors with a key, and revoke it when you're done. Your privacy is mathematically guaranteed.</p>
        </div>

        <!-- ORGANISATIONS -->
        <div class="gitehr-audience">
          <div class="gitehr-audience__header">
            <div class="gitehr-audience__icon">O</div>
            <h3>For Organisations</h3>
          </div>
          <p><strong>Secure and compliant.</strong> Eliminate the risk of massive centralized data breaches. Lower infrastructure costs.</p>
          <p>Native audit trails ensure compliance by default. Interoperate with legacy systems via standard HL7/FHIR adapters.</p>
        </div>

      </div>
    </div>
  </div>

  <!-- COMPARISON SECTION -->
  <div class="gitehr-band">
    <div class="gitehr-container">
      <h2>A Fundamental Shift</h2>
      <p class="gitehr-lead">Why moving from centralized databases to distributed version control changes everything.</p>
      
      <div class="gitehr-comparison">
        <div class="gitehr-comparison__col gitehr-comparison__col--legacy">
          <h3>Traditional Systems</h3>
          
          <div class="gitehr-comparison__item">
            <div class="gitehr-comparison__icon"></div>
            <div>
              <strong>Data Silos</strong><br>
              Records are trapped in one hospital's database.
            </div>
          </div>
          
          <div class="gitehr-comparison__item">
            <div class="gitehr-comparison__icon"></div>
            <div>
              <strong>Overwrite Updates</strong><br>
              "Correction" often deletes the previous value.
            </div>
          </div>

          <div class="gitehr-comparison__item">
            <div class="gitehr-comparison__icon"></div>
            <div>
              <strong>Institutional Ownership</strong><br>
              The hospital owns the data; patients must request it.
            </div>
          </div>

          <div class="gitehr-comparison__item">
            <div class="gitehr-comparison__icon"></div>
            <div>
              <strong>Complex Sync</strong><br>
              Fragile, expensive custom interfaces (HL7) to move data.
            </div>
          </div>

        </div>

        <div class="gitehr-comparison__col gitehr-comparison__col--gitehr">
          <h3>The GitEHR Way</h3>
          
          <div class="gitehr-comparison__item">
            <div class="gitehr-comparison__icon"></div>
            <div>
              <strong>Portable Files</strong><br>
              The record is a folder of files you can move anywhere.
            </div>
          </div>
          
          <div class="gitehr-comparison__item">
            <div class="gitehr-comparison__icon"></div>
            <div>
              <strong>Append-Only History</strong><br>
              Nothing is ever deleted. Full audit trail by default.
            </div>
          </div>

          <div class="gitehr-comparison__item">
            <div class="gitehr-comparison__icon"></div>
            <div>
              <strong>Patient Ownership</strong><br>
              The patient holds the repo; institutions are contributors.
            </div>
          </div>

          <div class="gitehr-comparison__item">
            <div class="gitehr-comparison__icon"></div>
            <div>
              <strong>Native Sync</strong><br>
              `git pull` and `git push` handle synchronization perfectly.
            </div>
          </div>

        </div>
      </div>
    </div>
  </div>

  <!-- HOW IT WORKS -->
  <div class="gitehr-band gitehr-band--alt">
    <div class="gitehr-container">
      <h2>How it works</h2>
      <div class="gitehr-steps">
        <div class="gitehr-step">
          <h3>Initialize</h3>
          <p>Create a GitEHR repository with a secure journal chain and structured folders.</p>
        </div>
        <div class="gitehr-step">
          <h3>Append</h3>
          <p>Each visit or update is a new commit, cryptographically linked to the history.</p>
        </div>
        <div class="gitehr-step">
          <h3>Sync</h3>
          <p>Collaborate via Git remotes. Clinicians push updates; patients pull the latest record.</p>
        </div>
      </div>
    </div>
  </div>

</div>
