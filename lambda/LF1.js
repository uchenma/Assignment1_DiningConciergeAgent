var sqsQueueUrl = "https://sqs.us-east-1.amazonaws.com/217015071650/yelp-restaurant-request";
var AWS = require('aws-sdk');
AWS.config.update({region: 'us-east-1'});


exports.handler = async (event) => {

    console.log(event);

    if(event.invocationSource == "FulfillmentCodeHook" && event.currentIntent.name == "GreetingIntent"){
        const response = {
            "dialogAction":{
                "fulfillmentState":"Fulfilled",
                "type":"Close",
                "message":{
                    "contentType":"PlainText",
                    "content": "Hi there, how can I help?"
                }
            }
        }
        return response;
    }else if (event.invocationSource == "FulfillmentCodeHook" && event.currentIntent.name == "ThankYouIntent"){
        const response = {
            "dialogAction":{
                "fulfillmentState":"Fulfilled",
                "type":"Close",
                "message":{
                    "contentType":"PlainText",
                    "content": "You’re welcome"
                }
            }
        }
        return response;
    } else if (event.invocationSource == "DialogCodeHook" && event.currentIntent.name == "DiningSuggestionsIntent_"){
        console.log("in the dining dialog codehook")
        
        const slots = event.currentIntent.slots;
        
        if (slots.DiningDate){
            // const today = new Date()
            // const yesterday = new Date(today)
            // yesterday.setDate(yesterday.getDate() - 1)
            
            if (new Date(slots.DiningDate) < new Date()){
                slots.DiningDate = null
                return {
                    "sessionAttributes": event.sessionAttributes,
                    dialogAction: {
                        "type": "ElicitSlot",
                        "intentName": event.currentIntent.name,
                        "slots": slots,
                        "slotToElicit": "DiningDate",
                        "message": { 
                            "contentType": "PlainText", 
                            "content": "Oops! Can only enter date in the future :(" },
                    },
                };
            }
        }
        
        if (slots.NumberOfPeople){
            if (slots.NumberOfPeople < 1){
                slots.NumberOfPeople = null;
                return {
                    "sessionAttributes": event.sessionAttributes,
                    dialogAction: {
                        "type": "ElicitSlot",
                        "intentName": event.currentIntent.name,
                        "slots": slots,
                        "slotToElicit": "NumberOfPeople",
                        "message": { 
                            "contentType": "PlainText", 
                            "content": "Invalid party size!" },
                    },
                };
            }
        }
        
        return {
            "sessionAttributes": event.sessionAttributes,
            "dialogAction": {
                "type": "Delegate",
                "slots": slots
            }
        }
        
    } else if (event.invocationSource == "FulfillmentCodeHook" && event.currentIntent.name == "DiningSuggestionsIntent_"){
        console.log("in the dining fullfilment codehook")
    
        var sqs = new AWS.SQS();
        
        var message = {
            phonenumber: event.currentIntent.slots.PhoneNumber,
            cuisine: event.currentIntent.slots.Cuisine,
            num_people: event.currentIntent.slots.NumberOfPeople,
            date_and_time: event.currentIntent.slots.DiningDate + " " + event.currentIntent.slots.DiningTime,
            location: event.currentIntent.slots.Location
        }
        
        var params = {
            MessageBody: JSON.stringify(message),
            QueueUrl: sqsQueueUrl
        };
        
        return sqs.sendMessage(params).promise()
        .then((data) => {
            console.log("Success");
            const response = {
                "dialogAction":{
                    "fulfillmentState":"Fulfilled",
                    "type":"Close",
                    "message":{
                        "contentType":"PlainText",
                        "content": "We'll send the restaurant recommendations shortly through an SMS :) "
                    }
                }
            }
            return response;
        }).catch((err) => {
            console.log("error while sending message to sqs: " + err);
        })
    }
};