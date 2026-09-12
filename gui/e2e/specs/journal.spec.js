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
      const journalCard = await $('.journal-panel');
      await journalCard.waitForDisplayed({ timeout: 10000 });
      const text = await journalCard.getText();
      expect(text).toContain('Journal');
      expect(text).toContain('Initial test entry');
    });

    it('should summarise the typed state the CLI holds', async () => {
      await browser.waitUntil(async () => (await $$('.state-card')).length === 4, {
        timeout: 10000,
        timeoutMsg: 'expected problem, medication, observation and vaccination cards',
      });

      let text = '';
      for (const card of await $$('.state-card')) {
        text += `${await card.getText()}\n`;
      }
      expect(text).toContain('Essential hypertension');
      expect(text).toContain('Ramipril');
      expect(text).toContain('Blood pressure');
      expect(text).toContain('Influenza');
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
      await browser.waitUntil(
        async () => await (await $('button*=Add')).isEnabled(),
        { timeout: 5000, timeoutMsg: 'Add button should become enabled' }
      );

      const addButton = await $('button*=Add');
      // Under xvfb the point-click is occasionally intercepted by the sticky
      // header; a DOM click is deterministic.
      await browser.execute((el) => el.click(), addButton);

      const newEntry = await $('div*=E2E test entry from WebDriverIO');
      await newEntry.waitForDisplayed({ timeout: 15000 });
      expect(await newEntry.isDisplayed()).toBe(true);

      await browser.waitUntil(async () => (await textarea.getValue()) === '', {
        timeout: 10000,
        timeoutMsg: 'textarea should be cleared after adding an entry',
      });
    });

    it('should keep an unfinished entry with the record it belongs to', async () => {
      const textarea = await $('textarea[placeholder*="new journal entry"]');
      await textarea.waitForDisplayed({ timeout: 10000 });
      await textarea.setValue('Unfinished note about the first patient');

      const backToList = await $('button*=Patient list');
      await backToList.waitForDisplayed({ timeout: 10000 });
      await browser.execute((el) => el.click(), backToList);

      // Open the other subject: the draft above must not have followed us.
      const openButtons = await $$('button*=Open');
      await browser.waitUntil(async () => (await $$('button*=Open')).length > 1, {
        timeout: 10000,
        timeoutMsg: 'expected two subjects in the Patient Index',
      });
      await browser.execute((el) => el.click(), openButtons[1]);

      const otherTextarea = await $('textarea[placeholder*="new journal entry"]');
      await otherTextarea.waitForDisplayed({ timeout: 10000 });
      await browser.waitUntil(async () => (await otherTextarea.getValue()) === '', {
        timeout: 10000,
        timeoutMsg: 'a draft must not carry across to another patient',
      });

      // Going back restores the draft to the record it was written about.
      const backAgain = await $('button*=Patient list');
      await browser.execute((el) => el.click(), backAgain);
      const reopen = await $$('button*=Open');
      await browser.execute((el) => el.click(), reopen[0]);

      const restored = await $('textarea[placeholder*="new journal entry"]');
      await restored.waitForDisplayed({ timeout: 10000 });
      await browser.waitUntil(
        async () =>
          (await restored.getValue()) === 'Unfinished note about the first patient',
        { timeout: 10000, timeoutMsg: 'the original draft should come back' }
      );
    });

    // Last, because paging the whole journal onto the screen pushes the entry
    // box out of view for anything that runs afterwards.
    it('should say how much of the journal it is showing, and page back', async () => {
      const journalCard = await $('.journal-panel');
      await journalCard.waitForDisplayed({ timeout: 10000 });

      // The record is longer than one page, so the count has to describe the
      // journal rather than the page: "25 of 29 entries", not "25 entries".
      const badge = await $('.journal-panel .mantine-Badge-label');
      await badge.waitForDisplayed({ timeout: 10000 });
      expect(await badge.getHTML()).toContain(' of ');

      // The oldest entry is off the first page until the reader asks for it.
      expect(await journalCard.getText()).not.toContain('Backfilled entry 1 for');

      const loadOlder = await $('button*=Load older entries');
      await loadOlder.waitForDisplayed({ timeout: 10000 });
      // Point-clicks are intercepted by the sticky header under xvfb.
      await browser.execute((el) => el.click(), loadOlder);

      const oldest = await $('div*=Backfilled entry 1 for paging coverage');
      await oldest.waitForDisplayed({ timeout: 15000 });
      expect(await oldest.isDisplayed()).toBe(true);
    });
  });
});