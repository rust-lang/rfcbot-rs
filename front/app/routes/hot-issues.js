import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';
import d3 from 'd3';
import cloud from 'npm:d3-cloud';

export default Ember.Route.extend({
  model() {
    const url = `${ENV.apiBaseURL}hot-issues`;
    return fetch(url).then(response => response.json());
  }
});
