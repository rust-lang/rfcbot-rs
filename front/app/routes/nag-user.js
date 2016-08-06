import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

export default Ember.Route.extend({
  model(params) {
    const url = `${ENV.apiBaseURL}nag/` + params.username;

    return fetch(url)
      .then(response => response.json())
      .then(({ username, full_name }) => ({
          user: username,
          fullName: full_name,
      }));
  }
});
