import Ember from 'ember';
import fetch from 'fetch';
import ENV from 'rust-dashboard/config/environment';

export default Ember.Route.extend({
  model: function() {
    const summary_url = `${ENV.apiBaseURL}releases`;
    return fetch(summary_url).then(({ nightlies }) => ({ nightlies }));
  }
});
