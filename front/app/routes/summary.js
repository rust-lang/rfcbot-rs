import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

export default Ember.Route.extend({
  model: function() {
    var summary_url = ENV.apiBaseURL + 'summary';
    return Ember.$.getJSON(summary_url);
  }
});
