import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

export default Ember.Route.extend({
  model() {
    const url = `${ENV.apiBaseURL}hot-issues`;
    return fetch(url).then(response => response.json());
  }
});
