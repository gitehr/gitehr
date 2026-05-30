describe('GitEHR GUI', () => {
  describe('Initial Load', () => {
    it('should show folder picker screen when no repo is detected', async () => {
      const title = await $('h3');
      await title.waitForDisplayed({ timeout: 10000 });
      const text = await title.getText();
      expect(text).toContain('No GitEHR Repository Detected');
    });

    it('should display the Open Repository button', async () => {
      const button = await $('button*=Open Repository');
      await button.waitForDisplayed({ timeout: 5000 });
      expect(await button.isDisplayed()).toBe(true);
    });

    it('should show helpful instructions', async () => {
      const instructions = await $('div*=Select a folder containing');
      await instructions.waitForDisplayed({ timeout: 5000 });
      const text = await instructions.getText();
      expect(text).toContain('GitEHR repository');
    });

    it('should display the GitEHR logo', async () => {
      const logo = await $('img[alt="GitEHR logo"]');
      await logo.waitForDisplayed({ timeout: 5000 });
      expect(await logo.isDisplayed()).toBe(true);
    });
  });
});
