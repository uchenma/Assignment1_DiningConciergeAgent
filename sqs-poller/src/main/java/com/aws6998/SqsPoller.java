package com.aws6998;

import java.util.List;

import javax.management.RuntimeErrorException;

import com.amazonaws.regions.Regions;
import com.amazonaws.services.lambda.AWSLambda;
import com.amazonaws.services.lambda.AWSLambdaClientBuilder;
import com.amazonaws.services.lambda.model.InvokeRequest;
import com.amazonaws.services.lambda.model.InvokeResult;
import com.amazonaws.services.lambda.runtime.Context;
import com.amazonaws.services.lambda.runtime.RequestHandler;
import com.amazonaws.services.sns.AmazonSNS;
import com.amazonaws.services.sns.AmazonSNSClient;
import com.amazonaws.services.sns.model.PublishRequest;
import com.amazonaws.services.sqs.AmazonSQS;
import com.amazonaws.services.sqs.AmazonSQSClientBuilder;
import com.amazonaws.services.sqs.model.Message;
import com.amazonaws.services.sqs.model.ReceiveMessageRequest;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.nio.charset.StandardCharsets;

class Empty {
}

class TextMessageToSend {
    public String phonenumber;
    public String message;
}

public class SqsPoller implements RequestHandler<Empty, String> {
    static String QUEUE_URL = "https://sqs.us-east-2.amazonaws.com/217015071650/yelp-restaurant-request";
    ObjectMapper objectMapper = new ObjectMapper();

    @Override
    public String handleRequest(Empty event, Context context) {
        final AmazonSQS sqs = AmazonSQSClientBuilder.defaultClient();
        List<Message> messages = sqs
                .receiveMessage((new ReceiveMessageRequest()).withQueueUrl(QUEUE_URL).withMaxNumberOfMessages(1))
                .getMessages();
        System.out.println("Number of messages: " + Integer.toString(messages.size()));

        AWSLambda awsLambda = AWSLambdaClientBuilder.standard().withRegion(Regions.US_EAST_2).build();
        for (Message message : messages) {
            InvokeRequest invokeRequest = new InvokeRequest().withFunctionName("restaurant-request-completer")
                    .withPayload(message.getBody());
            InvokeResult invokeResult = awsLambda.invoke(invokeRequest);
            String ans = new String(invokeResult.getPayload().array(), StandardCharsets.UTF_8);
            try {
                TextMessageToSend info = objectMapper.readValue(ans, TextMessageToSend.class);
                AmazonSNS sns = /* defaultClient does not support region */ (new AmazonSNSClient())
                        .withRegion(Regions.US_EAST_1);
               System.out.println("Sending text to " + info.phonenumber);
              System.out.println(info.message);

                sns.publish((new PublishRequest()).withPhoneNumber(info.phonenumber).withMessage(info.message));
                sqs.deleteMessage(QUEUE_URL, message.getReceiptHandle());
            } catch (Exception e) {
                System.out.println(ans);

                throw new RuntimeException(e);
            }
        }
        return "";

    }

}
