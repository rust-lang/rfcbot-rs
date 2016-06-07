import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

export default Ember.Route.extend({
  model: function() {
    const summary_url = `${ENV.apiBaseURL}releases`;
    return Ember.$.getJSON(summary_url)
      .then(metrics => {
        return {
          nightlies: metrics.nightlies
        };
      });
  }
});
