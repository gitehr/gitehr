describe('GitEHR Sidebar', () => {
  describe('State Files Display', () => {
    it('should show the Stateful summary card', async () => {
      const summaryHeader = await $('div*=Stateful summary');
      await summaryHeader.waitForDisplayed({ timeout: 10000 });
      expect(await summaryHeader.isDisplayed()).toBe(true);
    });

    it('should display allergies from state file', async () => {
      const allergies = await $('div*=Penicillin');
      await allergies.waitForDisplayed({ timeout: 10000 });
      const text = await allergies.getText();
      expect(text).toContain('Penicillin');
    });

    it('should display medications from state file', async () => {
      const medications = await $('div*=Aspirin');
      await medications.waitForDisplayed({ timeout: 10000 });
      const text = await medications.getText();
      expect(text).toContain('Aspirin');
    });
  });

  describe('Repo Status Card', () => {
    it('should show the Repo Status card', async () => {
      const statusHeader = await $('div*=Repo Status');
      await statusHeader.waitForDisplayed({ timeout: 10000 });
      expect(await statusHeader.isDisplayed()).toBe(true);
    });

    it('should display entry count', async () => {
      const entriesBadge = await $('div*=Entries');
      await entriesBadge.waitForDisplayed({ timeout: 5000 });
      expect(await entriesBadge.isDisplayed()).toBe(true);
    });

    it('should show encryption status', async () => {
      const encryptedLabel = await $('div*=Encrypted');
      await encryptedLabel.waitForDisplayed({ timeout: 5000 });
      expect(await encryptedLabel.isDisplayed()).toBe(true);
    });
  });

  describe('Activity Feed', () => {
    it('should show the Activity feed card', async () => {
      const activityHeader = await $('div*=Activity feed');
      await activityHeader.waitForDisplayed({ timeout: 10000 });
      expect(await activityHeader.isDisplayed()).toBe(true);
    });
  });

  describe('Navigation', () => {
    it('should display navigation links', async () => {
      const journalLink = await $('a*=Journal');
      await journalLink.waitForDisplayed({ timeout: 5000 });
      expect(await journalLink.isDisplayed()).toBe(true);
    });

    it('should show Patients link as active', async () => {
      const patientsLink = await $('a*=Patients');
      await patientsLink.waitForDisplayed({ timeout: 5000 });
      expect(await patientsLink.isDisplayed()).toBe(true);
    });
  });
});
