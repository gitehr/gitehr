// Copyright 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: CC-BY-SA-4.0

/*
 * With-repo E2E specs for the GUI shipped today (post-86b3995). Launching
 * against a Store root shows the Patient Index; each subject card has an
 * Open button that switches to the record view (patient-info header plus
 * Journal card). The old "Stateful summary" / "Repo Status" / "Activity
 * feed" sidebar screens are gone; R63 will build the five-screen MVP.
 */
describe('GitEHR Store and Record View', () => {
  describe('Patient Index', () => {
    it('should show the Patient Index screen', async () => {
      const title = await $('h2');
      await title.waitForDisplayed({ timeout: 10000 });
      expect(await title.getText()).toContain('Patient Index');
    });

    it('should list the e2e subject', async () => {
      const subjectName = await $('p*=e2e');
      await subjectName.waitForDisplayed({ timeout: 10000 });
      expect(await subjectName.isDisplayed()).toBe(true);
    });

    it('should show the NHS identifier seeded by the harness', async () => {
      const badge = await $('div*=NHS:');
      await badge.waitForDisplayed({ timeout: 10000 });
      expect(await badge.isDisplayed()).toBe(true);
    });
  });

  describe('Record view after Open', () => {
    it('should open the record and show typed allergies', async () => {
      const openButton = await $('button*=Open');
      await openButton.waitForDisplayed({ timeout: 10000 });
      await openButton.click();

      await $('.patient-info-allergies').waitForDisplayed({ timeout: 10000 });
      const badge = await $('.patient-info-allergies .mantine-Badge-label');
      await badge.waitForDisplayed({ timeout: 10000 });
      // wry/WebKitGTK returns '' for getText on this span; getHTML is reliable.
      expect(await badge.getHTML()).toContain('Penicillin');
    });

    it('should display the Journal card with the pre-seeded entry', async () => {
      const journalCard = await $('.panel-card');
      await journalCard.waitForDisplayed({ timeout: 10000 });
      const text = await journalCard.getText();
      expect(text).toContain('Journal');
      expect(text).toContain('Initial test entry');
    });

    it('should show the entry input textarea and Add button', async () => {
      const textarea = await $('textarea[placeholder*="new journal entry"]');
      await textarea.waitForDisplayed({ timeout: 5000 });
      expect(await textarea.isDisplayed()).toBe(true);

      const addButton = await $('button*=Add');
      await addButton.waitForDisplayed({ timeout: 5000 });
      expect(await addButton.isDisplayed()).toBe(true);
    });

    it('should add an entry when Add is clicked, then clear the textarea', async () => {
      const textarea = await $('textarea[placeholder*="new journal entry"]');
      await textarea.waitForDisplayed({ timeout: 5000 });
      await textarea.setValue('E2E test entry from WebDriverIO');

      const addButton = await $('button*=Add');
      await addButton.click();

      const newEntry = await $('div*=E2E test entry from WebDriverIO');
      await newEntry.waitForDisplayed({ timeout: 15000 });
      expect(await newEntry.isDisplayed()).toBe(true);

      await browser.waitUntil(async () => (await textarea.getValue()) === '', {
        timeout: 10000,
        timeoutMsg: 'textarea should be cleared after adding an entry',
      });
    });
  });
});