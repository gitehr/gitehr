describe('GitEHR Journal', () => {
  describe('Journal View', () => {
    it('should display the Journal card header', async () => {
      const journalHeader = await $('div*=Journal');
      await journalHeader.waitForDisplayed({ timeout: 10000 });
      expect(await journalHeader.isDisplayed()).toBe(true);
    });

    it('should show the entry input textarea', async () => {
      const textarea = await $('textarea[placeholder*="new journal entry"]');
      await textarea.waitForDisplayed({ timeout: 5000 });
      expect(await textarea.isDisplayed()).toBe(true);
    });

    it('should have an Add button', async () => {
      const addButton = await $('button*=Add');
      await addButton.waitForDisplayed({ timeout: 5000 });
      expect(await addButton.isDisplayed()).toBe(true);
    });

    it('should display existing journal entries', async () => {
      const entryCard = await $('div*=Initial test entry');
      await entryCard.waitForDisplayed({ timeout: 10000 });
      const text = await entryCard.getText();
      expect(text).toContain('Initial test entry');
    });
  });

  describe('Adding Entries', () => {
    it('should allow typing in the textarea', async () => {
      const textarea = await $('textarea[placeholder*="new journal entry"]');
      await textarea.waitForDisplayed({ timeout: 5000 });
      await textarea.setValue('E2E test entry from WebDriverIO');
      const value = await textarea.getValue();
      expect(value).toBe('E2E test entry from WebDriverIO');
    });

    it('should enable Add button when text is entered', async () => {
      const addButton = await $('button*=Add');
      await addButton.waitForDisplayed({ timeout: 5000 });
      const isEnabled = await addButton.isEnabled();
      expect(isEnabled).toBe(true);
    });

    it('should add entry when Add button is clicked', async () => {
      const addButton = await $('button*=Add');
      await addButton.click();
      
      await browser.pause(2000);
      
      const newEntry = await $('div*=E2E test entry from WebDriverIO');
      await newEntry.waitForDisplayed({ timeout: 10000 });
      expect(await newEntry.isDisplayed()).toBe(true);
    });

    it('should clear textarea after adding entry', async () => {
      const textarea = await $('textarea[placeholder*="new journal entry"]');
      const value = await textarea.getValue();
      expect(value).toBe('');
    });
  });
});
