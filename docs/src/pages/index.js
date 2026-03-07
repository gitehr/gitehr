// SPDX-License-Identifier: CC-BY-SA-4.0
import React from 'react';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';

import styles from './index.module.css';

export default function Home() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={siteConfig.title}
      description="Decentralised, lossless health records powered by Git, cryptography, and open standards."
    >
      <main className="gitehr-landing">

        {/* HERO SECTION */}
        <div className="gitehr-band gitehr-band--hero">
          <div className="gitehr-container">
            <div className="gitehr-hero">
              <div className="gitehr-hero__content">
                <div className="gitehr-kicker">Open Source Standard</div>
                <h1 className="gitehr-hero__title">
                  Decentralised, lossless <span>health records</span>
                </h1>
                <p className="gitehr-hero__subtitle">
                  GitEHR is an open source platform powered by Git, cryptography, and open standards.
                  It keeps every clinical change auditable, portable, and owned by the patient.
                </p>
                <div className="gitehr-hero__actions">
                  <Link to="/getting-started" className="gitehr-button gitehr-button--primary">
                    Get Started
                  </Link>
                  <Link
                    href="https://github.com/gitehr/gitehr"
                    className="gitehr-button gitehr-button--secondary"
                  >
                    View on GitHub
                  </Link>
                </div>
              </div>
              <div className="gitehr-hero__image">
                <img src="/img/hero-illustration.svg" alt="GitEHR architecture illustration" />
              </div>
            </div>
          </div>
        </div>

        {/* GETTING STARTED CALLOUT */}
        <div className="gitehr-band gitehr-band--cta">
          <div className="gitehr-container">
            <div className="gitehr-cta">
              <div className="gitehr-cta__content">
                <div className="gitehr-kicker">Developer preview</div>
                <h3>Start with the GUI and repository basics</h3>
                <p>
                  Use the Getting Started guide to run the GUI, open a repository, and learn the
                  on-disk layout.
                </p>
              </div>
              <div className="gitehr-cta__actions">
                <Link to="/getting-started" className="gitehr-button gitehr-button--primary">
                  Read the guide
                </Link>
                <Link
                  to="/guides/gui-walkthrough"
                  className="gitehr-button gitehr-button--secondary"
                >
                  GUI walkthrough
                </Link>
              </div>
            </div>
          </div>
        </div>

        {/* VALUE PROP SECTION */}
        <div className="gitehr-band">
          <div className="gitehr-container">
            <h2>The Health Record, Reimagined</h2>
            <p className="gitehr-lead">
              We replaced the siloed database with a distributed ledger. The result is a system that
              is naturally secure, offline-first, and infinitely portable.
            </p>
            <div className="gitehr-grid gitehr-grid--3">
              <div className="gitehr-card">
                <h3>Patient Owned</h3>
                <p>
                  Records live with the patient, not the institution. Share them across providers,
                  borders, and decades without rebuilding the data.
                </p>
              </div>
              <div className="gitehr-card">
                <h3>Cryptographic Trust</h3>
                <p>
                  Every entry links to the last one, forming an append-only chain that is
                  tamper-evident and mathematically verifiable.
                </p>
              </div>
              <div className="gitehr-card">
                <h3>Offline First</h3>
                <p>
                  Git-powered repositories work without constant connectivity. Sync seamlessly
                  whenever a secure link is available.
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* AUDIENCE SECTION */}
        <div className="gitehr-band gitehr-band--alt">
          <div className="gitehr-container">
            <h2>Built for the entire ecosystem</h2>
            <div className="gitehr-grid gitehr-grid--3">

              <div className="gitehr-audience">
                <div className="gitehr-audience__header">
                  <div className="gitehr-audience__icon">C</div>
                  <h3>For Clinicians</h3>
                </div>
                <p>
                  <strong>See the whole picture.</strong> Access a patient's complete longitudinal
                  history, not just what's in your local system.
                </p>
                <p>
                  GitEHR provides a unified view of care across specialists and facilities, reducing
                  medical errors and redundant testing.
                </p>
              </div>

              <div className="gitehr-audience">
                <div className="gitehr-audience__header">
                  <div className="gitehr-audience__icon">P</div>
                  <h3>For Patients</h3>
                </div>
                <p>
                  <strong>Your health, in your hands.</strong> You hold the master copy of your
                  record. No more requesting faxed transfers.
                </p>
                <p>
                  Grant temporary access to new doctors with a key, and revoke it when you're done.
                  Your privacy is mathematically guaranteed.
                </p>
              </div>

              <div className="gitehr-audience">
                <div className="gitehr-audience__header">
                  <div className="gitehr-audience__icon">O</div>
                  <h3>For Organisations</h3>
                </div>
                <p>
                  <strong>Secure and compliant.</strong> Eliminate the risk of massive centralized
                  data breaches. Lower infrastructure costs.
                </p>
                <p>
                  Native audit trails ensure compliance by default. Interoperate with legacy systems
                  via standard HL7/FHIR adapters.
                </p>
              </div>

            </div>
          </div>
        </div>

        {/* COMPARISON SECTION */}
        <div className="gitehr-band">
          <div className="gitehr-container">
            <h2>A Fundamental Shift</h2>
            <p className="gitehr-lead">
              Why moving from centralized databases to distributed version control changes everything.
            </p>
            <div className="gitehr-comparison">
              <div className="gitehr-comparison__col gitehr-comparison__col--legacy">
                <h3>Traditional Systems</h3>

                <div className="gitehr-comparison__item">
                  <div className="gitehr-comparison__icon"></div>
                  <div>
                    <strong>Data Silos</strong>
                    <br />
                    Records are trapped in one hospital's database.
                  </div>
                </div>

                <div className="gitehr-comparison__item">
                  <div className="gitehr-comparison__icon"></div>
                  <div>
                    <strong>Overwrite Updates</strong>
                    <br />
                    "Correction" often deletes the previous value.
                  </div>
                </div>

                <div className="gitehr-comparison__item">
                  <div className="gitehr-comparison__icon"></div>
                  <div>
                    <strong>Institutional Ownership</strong>
                    <br />
                    The hospital owns the data; patients must request it.
                  </div>
                </div>

                <div className="gitehr-comparison__item">
                  <div className="gitehr-comparison__icon"></div>
                  <div>
                    <strong>Complex Sync</strong>
                    <br />
                    Fragile, expensive custom interfaces (HL7) to move data.
                  </div>
                </div>
              </div>

              <div className="gitehr-comparison__col gitehr-comparison__col--gitehr">
                <h3>The GitEHR Way</h3>

                <div className="gitehr-comparison__item">
                  <div className="gitehr-comparison__icon"></div>
                  <div>
                    <strong>Portable Files</strong>
                    <br />
                    The record is a folder of files you can move anywhere.
                  </div>
                </div>

                <div className="gitehr-comparison__item">
                  <div className="gitehr-comparison__icon"></div>
                  <div>
                    <strong>Append-Only History</strong>
                    <br />
                    Nothing is ever deleted. Full audit trail by default.
                  </div>
                </div>

                <div className="gitehr-comparison__item">
                  <div className="gitehr-comparison__icon"></div>
                  <div>
                    <strong>Patient Ownership</strong>
                    <br />
                    The patient holds the repo; institutions are contributors.
                  </div>
                </div>

                <div className="gitehr-comparison__item">
                  <div className="gitehr-comparison__icon"></div>
                  <div>
                    <strong>Native Sync</strong>
                    <br />
                    <code>git pull</code> and <code>git push</code> handle synchronization
                    perfectly.
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* HOW IT WORKS */}
        <div className="gitehr-band gitehr-band--alt">
          <div className="gitehr-container">
            <h2>How it works</h2>
            <div className="gitehr-steps">
              <div className="gitehr-step">
                <h3>Initialize</h3>
                <p>Create a GitEHR repository with a secure journal chain and structured folders.</p>
              </div>
              <div className="gitehr-step">
                <h3>Append</h3>
                <p>
                  Each visit or update is a new commit, cryptographically linked to the history.
                </p>
              </div>
              <div className="gitehr-step">
                <h3>Sync</h3>
                <p>
                  Collaborate via Git remotes. Clinicians push updates; patients pull the latest
                  record.
                </p>
              </div>
            </div>
          </div>
        </div>

      </main>
    </Layout>
  );
}
