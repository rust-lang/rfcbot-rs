import Ember from 'ember';
import fetch from 'fetch';
import ENV from 'rust-dashboard/config/environment';

export default Ember.Route.extend({
  model() {
    const summary_url = `${ENV.apiBaseURL}nightlies`;
    return fetch(summary_url)
      .then(response => response.json());
  }
});
