import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

export default Ember.Route.extend({
  model: function() {
    const url = `${ENV.apiBaseURL}nag/users`;
    return Ember.$.getJSON(url)
      .then(members => {
        return {
          users: members
        };
      });
  }
});
