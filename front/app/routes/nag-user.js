import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

export default Ember.Route.extend({
  model(params) {

    const url = `${ENV.apiBaseURL}nag/` + params.username;
    return Ember.$.getJSON(url)
      .then(members => {
        return {
          user: params.username,
          fullName: members.full_name,
        };
      });
  }
});
