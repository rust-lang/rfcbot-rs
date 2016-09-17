import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

export default Ember.Route.extend({
  model(params) {
    const url = `${ENV.apiBaseURL}nag/` + params.username;

    return fetch(url)
      .then(response => response.json())
      .then(response => ({
          user: response[0],
          fcps: response[1],
      }));
  }
});
