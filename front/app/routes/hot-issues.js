import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

export default Ember.Route.extend({
  model: function() {
    const url = `${ENV.apiBaseURL}hot-issues`;
    return Ember.$.getJSON(url)
      .then(issues => {
        return {
          issues: issues
        };
      });
  }
});
