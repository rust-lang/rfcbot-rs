import Ember from 'ember';

export default Ember.Component.extend({
    opts: {
        chart: {
            height: 250
        },
        navigator: {
            enabled: false
        },
        scrollbar: {
            enabled: false
        },
        rangeSelector: {
            enabled: false
        }
    }
});
